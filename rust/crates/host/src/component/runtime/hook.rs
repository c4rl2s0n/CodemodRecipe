//! Dry-run / apply component hooks (identify/parse/relate/verify/…).

use std::collections::BTreeMap;
use std::rc::Rc;

use codemod_recipe_lua::new_sandbox_with_require;

use crate::component::runtime::context::{
    invoke_component_hook, invoke_component_hook_with_facade, make_host,
    try_invoke_component_hook, ComponentInvokeRequest, ComponentRunContext,
    ComponentScaffoldRequest,
};
use crate::component::runtime::finalize::{finalize_host, hook_to_json, take_transcript};
use crate::component::runtime::operations::invoke_operation;
use crate::component::runtime::scaffold::invoke_scaffold;
use crate::component::scan::component_scan;
use crate::component::types::ComponentInvokeResponse;
use crate::lua_runtime::RunKind;
use crate::recipe::registry::RecipeRegistry;

pub struct ComponentHookRequest {
    pub component_id: String,
    pub hook: String,
    pub kind: RunKind,
    pub locator: Option<serde_json::Value>,
    pub instance: Option<serde_json::Value>,
    pub operation: Option<String>,
    pub args: BTreeMap<String, serde_json::Value>,
}

/// Unified dry-run / apply entry for component hooks (identify/parse/verify/toSummary/scaffold/operation).
pub fn invoke_hook(
    registry: &mut RecipeRegistry,
    request: ComponentHookRequest,
) -> ComponentInvokeResponse {
    match request.hook.as_str() {
        "scaffold" => invoke_scaffold(
            registry,
            ComponentScaffoldRequest {
                component_id: request.component_id,
                args: request.args,
                kind: request.kind,
            },
        ),
        "operation" => {
            let instance_id = request
                .instance
                .as_ref()
                .and_then(|v| v.get("id"))
                .and_then(|v| v.as_str())
                .map(str::to_string);
            let Some(instance_id) = instance_id else {
                return ComponentInvokeResponse::fail(
                    "hook=operation requires instance with id (from scan or prior dry-run)",
                );
            };
            let Some(operation) = request.operation else {
                return ComponentInvokeResponse::fail("hook=operation requires operation name");
            };
            invoke_operation(
                registry,
                ComponentInvokeRequest {
                    instance_id,
                    operation,
                    args: request.args,
                    kind: request.kind,
                },
            )
        }
        "identify" | "parse" | "verify" | "toSummary" | "relate" => {
            invoke_read_hook(registry, request)
        }
        other => ComponentInvokeResponse::fail(format!(
            "unknown component hook {other:?}; expected identify|parse|relate|verify|toSummary|scaffold|operation"
        )),
    }
}

fn invoke_read_hook(
    registry: &mut RecipeRegistry,
    request: ComponentHookRequest,
) -> ComponentInvokeResponse {
    let Some(component) = registry.get_component(&request.component_id).cloned() else {
        return ComponentInvokeResponse::fail(format!(
            "component not found: {}",
            request.component_id
        ));
    };
    if request.hook == "identify" && !component.has_identify {
        return ComponentInvokeResponse::fail(format!(
            "component {} has no identify hook",
            request.component_id
        ));
    }
    if request.hook == "toSummary" && !component.has_to_summary {
        return ComponentInvokeResponse::fail(format!(
            "component {} has no toSummary hook",
            request.component_id
        ));
    }
    if request.hook == "relate" && !component.has_relate {
        return ComponentInvokeResponse::fail(format!(
            "component {} has no relate hook",
            request.component_id
        ));
    }
    // verify is optional: omission ≡ no judgment.
    if request.hook == "verify" && !component.has_verify {
        return ComponentInvokeResponse {
            ok: true,
            error: None,
            result: Some(serde_json::json!([])),
            instance: None,
            findings: None,
            files: None,
            transcript: None,
            logs: None,
            applied: None,
            preview_token: None,
        };
    }

    let arg = match request.hook.as_str() {
        "identify" => None,
        "parse" => {
            let Some(locator) = request.locator else {
                return ComponentInvokeResponse::fail("hook=parse requires locator");
            };
            Some(locator)
        }
        "verify" | "toSummary" | "relate" => {
            let Some(instance) = request.instance else {
                return ComponentInvokeResponse::fail(format!(
                    "hook={} requires instance",
                    request.hook
                ));
            };
            Some(instance)
        }
        _ => None,
    };

    let ctx = ComponentRunContext::from_registry(registry, request.kind);
    let host = match make_host(&ctx, request.args.clone()) {
        Ok(h) => h,
        Err(message) => return ComponentInvokeResponse::fail(message),
    };
    let lua = match new_sandbox_with_require(&ctx.codemod_root) {
        Ok(l) => l,
        Err(e) => return ComponentInvokeResponse::fail(e.to_string()),
    };

    // Facade-aware hooks (relate/verify) run against the cached scan forest;
    // with no cache yet, dry-run triggers a full scan first.
    let facade = if matches!(request.hook.as_str(), "verify" | "relate") {
        if registry.component_scan_cache().is_none() {
            let scan = component_scan(registry);
            if !scan.ok {
                return ComponentInvokeResponse::fail(
                    scan.error
                        .unwrap_or_else(|| "component scan failed".to_string()),
                );
            }
        }
        let Some(cache) = registry.component_scan_cache().cloned() else {
            return ComponentInvokeResponse::fail("component scan cache unavailable");
        };
        let flat = crate::component::relations::flatten_instances(&cache.instances);
        let facade = if request.hook == "relate" {
            crate::component::relations::build_index_facade(&lua, &flat)
        } else {
            crate::component::relations::build_graph_facade(&lua, &flat, &cache.edges)
        };
        match facade {
            Ok(f) => Some(f),
            Err(message) => {
                return ComponentInvokeResponse::fail(message)
                    .with_transcript(take_transcript(&host));
            }
        }
    } else {
        None
    };

    let call = if let Some(facade) = facade {
        let instance = arg.expect("facade hooks require instance");
        invoke_component_hook_with_facade(
            &lua,
            &component,
            &request.hook,
            Rc::clone(&host),
            instance,
            &facade,
        )
        .and_then(|opt| opt.ok_or_else(|| format!("component {} hook missing", request.hook)))
    } else if request.hook == "toSummary" {
        try_invoke_component_hook(&lua, &ctx, &component, "toSummary", Rc::clone(&host), arg)
            .and_then(|opt| {
                opt.ok_or_else(|| {
                    format!("component {} has no toSummary hook", request.component_id)
                })
            })
    } else {
        invoke_component_hook(&lua, &ctx, &component, &request.hook, Rc::clone(&host), arg)
    };

    match call {
        Ok(value) => {
            let result_json = match hook_to_json(&value) {
                Ok(v) => v,
                Err(e) => {
                    return ComponentInvokeResponse::fail(e).with_transcript(take_transcript(&host));
                }
            };
            let parts = match finalize_host(&host, request.kind) {
                Ok(parts) => parts,
                Err(e) => {
                    let mut resp = ComponentInvokeResponse::fail(e)
                        .with_transcript(take_transcript(&host));
                    resp.result = Some(result_json);
                    return resp;
                }
            };
            ComponentInvokeResponse {
                ok: true,
                error: None,
                result: Some(result_json),
                instance: None,
                findings: None,
                files: parts.files,
                transcript: None,
                logs: None,
                applied: parts.applied,
                preview_token: None,
            }
            .with_transcript(Some(parts.transcript))
        }
        Err(message) => {
            ComponentInvokeResponse::fail(message).with_transcript(take_transcript(&host))
        }
    }
}

