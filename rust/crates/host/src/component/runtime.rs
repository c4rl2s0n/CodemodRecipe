use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;

use codemod_recipe_engine::{LanguageRegistry, RegistryConfig};
use codemod_recipe_lua::json_lua::{json_to_lua, lua_to_json};
use codemod_recipe_lua::model::ComponentDef;
use codemod_recipe_lua::{eval_return_table, new_sandbox};
use mlua::{Function, Lua, Table, Value};

use crate::component::scan::{
    component_incremental_scan, component_scan, scan_component, ComponentIncrementalScanRequest,
};
use crate::component::types::{
    ComponentFinding, ComponentInstance, ComponentInvokeResponse,
};
use crate::engine::working_tree::WorkingTree;
use crate::host_api::context::{HostInner, RunKind, SharedHost};
use crate::lua_runtime::{args_table_from_map, build_ctx};
use crate::preview::diff::build_file_preview_from_change;
use crate::recipe::registry::RecipeRegistry;
use crate::sandbox::path_sandbox::PathSandbox;

pub struct ComponentRunContext {
    pub kind: RunKind,
    pub maps: BTreeMap<String, BTreeMap<String, String>>,
    pub vars: BTreeMap<String, BTreeMap<String, String>>,
    pub queries: BTreeMap<String, String>,
    pub recipes: BTreeMap<String, codemod_recipe_lua::model::Recipe>,
    pub components: BTreeMap<String, ComponentDef>,
    pub workspace_root: PathBuf,
    pub codemod_root: PathBuf,
    pub language_config: RegistryConfig,
}

impl ComponentRunContext {
    pub fn from_registry(registry: &RecipeRegistry, kind: RunKind) -> Self {
        Self {
            kind,
            maps: registry.maps_by_id().clone(),
            vars: registry.vars_by_id().clone(),
            queries: registry.queries_by_id().clone(),
            recipes: registry.recipes_map().clone(),
            components: registry.components().clone(),
            workspace_root: registry.workspace_root.clone(),
            codemod_root: registry.codemod_root().to_path_buf(),
            language_config: registry.language_registry.config().clone(),
        }
    }
}

pub struct ComponentInvokeRequest {
    pub instance_id: String,
    pub operation: String,
    pub args: BTreeMap<String, serde_json::Value>,
    pub kind: RunKind,
}

pub struct ComponentScaffoldRequest {
    pub component_id: String,
    pub args: BTreeMap<String, serde_json::Value>,
    pub kind: RunKind,
}

pub fn invoke_component_hook(
    lua: &Lua,
    _ctx: &ComponentRunContext,
    component: &ComponentDef,
    hook: &str,
    host: SharedHost,
    arg: Option<serde_json::Value>,
) -> Result<Value, String> {
    let ctx_table = build_ctx(lua, host)?;
    let name = component
        .path
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or(component.id.as_str());
    let module = eval_return_table(lua, &component.source, name).map_err(|e| e.to_string())?;
    let func: Function = module.get(hook).map_err(|_| {
        if hook == "readSource" {
            format!("component {} has no readSource hook", component.id)
        } else {
            format!("component {} missing required hook {hook}", component.id)
        }
    })?;
    let arg_lua = match arg {
        Some(value) => Some(json_to_lua(lua, &value)?),
        None => None,
    };
    match arg_lua {
        Some(value) => func
            .call::<Value>((ctx_table, value))
            .map_err(|e| format!("component {hook} failed: {e}")),
        None => func
            .call::<Value>(ctx_table)
            .map_err(|e| format!("component {hook} failed: {e}")),
    }
}

/// Calls an optional component hook; returns `None` when the hook is absent.
pub(crate) fn try_invoke_component_hook(
    lua: &Lua,
    ctx: &ComponentRunContext,
    component: &ComponentDef,
    hook: &str,
    host: SharedHost,
    arg: Option<serde_json::Value>,
) -> Result<Option<Value>, String> {
    let name = component
        .path
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or(component.id.as_str());
    let module = eval_return_table(lua, &component.source, name).map_err(|e| e.to_string())?;
    if module.get::<Option<Function>>(hook).map_err(|e| e.to_string())?.is_none() {
        return Ok(None);
    }
    invoke_component_hook(lua, ctx, component, hook, host, arg).map(Some)
}

pub(crate) fn make_host(
    ctx: &ComponentRunContext,
    args: BTreeMap<String, serde_json::Value>,
) -> Result<SharedHost, String> {
    Ok(Rc::new(Mutex::new(HostInner {
        kind: ctx.kind,
        tree: WorkingTree::new(),
        sandbox: PathSandbox::new(ctx.workspace_root.clone()),
        language_registry: LanguageRegistry::with_config(ctx.language_config.clone()),
        args,
        maps: ctx.maps.clone(),
        vars: ctx.vars.clone(),
        queries: ctx.queries.clone(),
        recipes: ctx.recipes.clone(),
        components: ctx.components.clone(),
        component_stack: Vec::new(),
        recipe_path: None,
        codemod_root: ctx.codemod_root.clone(),
        transcript: Vec::new(),
        saw_post_exec: false,
        op_count: 0,
    })))
}

pub fn invoke_operation(
    registry: &mut RecipeRegistry,
    request: ComponentInvokeRequest,
) -> ComponentInvokeResponse {
    if registry.stack().is_none() {
        return ComponentInvokeResponse {
            ok: false,
            error: Some("stack manifest is required for component operations".to_string()),
            instance: None,
            findings: None,
            files: None,
            transcript: None,
            applied: None,
        };
    }
    let ctx = ComponentRunContext::from_registry(registry, request.kind);
    let scan = crate::component::scan::component_scan(registry);
    if !scan.ok {
        return ComponentInvokeResponse {
            ok: false,
            error: scan.error,
            instance: None,
            findings: None,
            files: None,
            transcript: None,
            applied: None,
        };
    }
    let instances = scan.instances.unwrap_or_default();
    let Some(target) = instances.iter().find(|i| i.id == request.instance_id) else {
        return ComponentInvokeResponse {
            ok: false,
            error: Some(format!("instance not found: {}", request.instance_id)),
            instance: None,
            findings: None,
            files: None,
            transcript: None,
            applied: None,
        };
    };
    let component_id = target.component_id.clone();
    let Some(component) = registry.get_component(&component_id).cloned() else {
        return ComponentInvokeResponse {
            ok: false,
            error: Some(format!("component not found: {component_id}")),
            instance: None,
            findings: None,
            files: None,
            transcript: None,
            applied: None,
        };
    };
    if !component.operations.contains(&request.operation) {
        return ComponentInvokeResponse {
            ok: false,
            error: Some(format!(
                "unknown operation {} on component {}",
                request.operation, component_id
            )),
            instance: None,
            findings: None,
            files: None,
            transcript: None,
            applied: None,
        };
    }
    let host = match make_host(&ctx, request.args.clone()) {
        Ok(h) => h,
        Err(message) => {
            return ComponentInvokeResponse {
                ok: false,
                error: Some(message),
                instance: None,
                findings: None,
                files: None,
                transcript: None,
                applied: None,
            };
        }
    };
    let lua = match new_sandbox() {
        Ok(l) => l,
        Err(e) => {
            return ComponentInvokeResponse {
                ok: false,
                error: Some(e.to_string()),
                instance: None,
                findings: None,
                files: None,
                transcript: None,
                applied: None,
            };
        }
    };
    let result = (|| -> Result<ComponentInvokeResponse, String> {
        let ctx_table = build_ctx(&lua, Rc::clone(&host))?;
        let name = component
            .path
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or(component.id.as_str());
        let module = eval_return_table(&lua, &component.source, name).map_err(|e| e.to_string())?;
        let ops: Table = module
            .get("operations")
            .map_err(|e| format!("operations table missing: {e}"))?;
        let op_tbl: Table = ops
            .get(request.operation.as_str())
            .map_err(|_| format!("operation {} not found", request.operation))?;
        let apply: Function = op_tbl
            .get("apply")
            .map_err(|e| format!("operation apply missing: {e}"))?;
        let instance_tbl = json_to_lua(
            &lua,
            &serde_json::json!({
                "id": target.id,
                "componentId": target.component_id,
                "locator": target.locator,
                "node": target.node,
            }),
        )?;
        let args_tbl = args_table_from_map(&lua, &request.args)?;
        apply
            .call::<()>((
                ctx_table,
                args_tbl,
                instance_tbl,
            ))
            .map_err(|e| format!("operation apply failed: {e}"))?;

        let mut inner = host.lock().map_err(|e| e.to_string())?;
        let applied = if request.kind == RunKind::Apply {
            Some(inner.flush_to_disk()?)
        } else {
            None
        };
        let changes = std::mem::replace(&mut inner.tree, WorkingTree::new()).finalize();
        let transcript = std::mem::take(&mut inner.transcript);
        drop(inner);

        let rescanned =
            component_scan_after_operation(registry, &target.id, &changes)?;
        let files = if changes.is_empty() {
            None
        } else {
            Some(
                changes
                    .iter()
                    .filter_map(|c| build_file_preview_from_change(c, false, None).ok())
                    .collect(),
            )
        };
        Ok(ComponentInvokeResponse {
            ok: true,
            error: None,
            instance: rescanned.instance,
            findings: rescanned.findings,
            files,
            transcript: Some(transcript),
            applied,
        })
    })();
    match result {
        Ok(resp) => resp,
        Err(message) => ComponentInvokeResponse {
            ok: false,
            error: Some(message),
            instance: None,
            findings: None,
            files: None,
            transcript: host
                .lock()
                .ok()
                .map(|mut inner| std::mem::take(&mut inner.transcript))
                .filter(|t| !t.is_empty()),
            applied: None,
        },
    }
}

pub fn invoke_scaffold(
    registry: &mut RecipeRegistry,
    request: ComponentScaffoldRequest,
) -> ComponentInvokeResponse {
    if registry.stack().is_none() {
        return ComponentInvokeResponse {
            ok: false,
            error: Some("stack manifest is required for component scaffold".to_string()),
            instance: None,
            findings: None,
            files: None,
            transcript: None,
            applied: None,
        };
    }
    let Some(component) = registry.get_component(&request.component_id).cloned() else {
        return ComponentInvokeResponse {
            ok: false,
            error: Some(format!("component not found: {}", request.component_id)),
            instance: None,
            findings: None,
            files: None,
            transcript: None,
            applied: None,
        };
    };
    if !component.has_scaffold {
        return ComponentInvokeResponse {
            ok: false,
            error: Some(format!(
                "component {} has no scaffold entry",
                request.component_id
            )),
            instance: None,
            findings: None,
            files: None,
            transcript: None,
            applied: None,
        };
    }

    // Ensure cache exists so post-scaffold detectChanges can glob-diff new roots.
    let _ = component_scan(registry);

    let ctx = ComponentRunContext::from_registry(registry, request.kind);
    let host = match make_host(&ctx, request.args.clone()) {
        Ok(h) => h,
        Err(message) => {
            return ComponentInvokeResponse {
                ok: false,
                error: Some(message),
                instance: None,
                findings: None,
                files: None,
                transcript: None,
                applied: None,
            };
        }
    };
    let lua = match new_sandbox() {
        Ok(l) => l,
        Err(e) => {
            return ComponentInvokeResponse {
                ok: false,
                error: Some(e.to_string()),
                instance: None,
                findings: None,
                files: None,
                transcript: None,
                applied: None,
            };
        }
    };
    let result = (|| -> Result<ComponentInvokeResponse, String> {
        let ctx_table = build_ctx(&lua, Rc::clone(&host))?;
        let name = component
            .path
            .as_ref()
            .and_then(|p| p.to_str())
            .unwrap_or(component.id.as_str());
        let module = eval_return_table(&lua, &component.source, name).map_err(|e| e.to_string())?;
        let scaffold: Table = module
            .get("scaffold")
            .map_err(|e| format!("scaffold table missing: {e}"))?;
        let apply: Function = scaffold
            .get("apply")
            .map_err(|e| format!("scaffold apply missing: {e}"))?;
        let args_tbl = args_table_from_map(&lua, &request.args)?;
        apply
            .call::<()>((ctx_table, args_tbl))
            .map_err(|e| format!("scaffold apply failed: {e}"))?;

        let mut inner = host.lock().map_err(|e| e.to_string())?;
        let applied = if request.kind == RunKind::Apply {
            Some(inner.flush_to_disk()?)
        } else {
            None
        };
        let changes = std::mem::replace(&mut inner.tree, WorkingTree::new()).finalize();
        let transcript = std::mem::take(&mut inner.transcript);
        drop(inner);

        let scan = component_incremental_scan(
            registry,
            ComponentIncrementalScanRequest {
                dirty_paths: changes.iter().map(|c| c.path().to_string()).collect(),
                detect_changes: true,
                ..Default::default()
            },
        );
        if !scan.ok {
            return Err(scan.error.unwrap_or_else(|| "rescan failed".to_string()));
        }
        let files = if changes.is_empty() {
            None
        } else {
            Some(
                changes
                    .iter()
                    .filter_map(|c| build_file_preview_from_change(c, false, None).ok())
                    .collect(),
            )
        };
        Ok(ComponentInvokeResponse {
            ok: true,
            error: None,
            instance: None,
            findings: scan.findings,
            files,
            transcript: Some(transcript),
            applied,
        })
    })();
    match result {
        Ok(resp) => resp,
        Err(message) => ComponentInvokeResponse {
            ok: false,
            error: Some(message),
            instance: None,
            findings: None,
            files: None,
            transcript: host
                .lock()
                .ok()
                .map(|mut inner| std::mem::take(&mut inner.transcript))
                .filter(|t| !t.is_empty()),
            applied: None,
        },
    }
}

struct RescanResult {
    instance: Option<ComponentInstance>,
    findings: Option<Vec<ComponentFinding>>,
}

fn component_scan_after_operation(
    registry: &mut RecipeRegistry,
    instance_id: &str,
    changes: &[codemod_recipe_core::apply::file_change::FileChange],
) -> Result<RescanResult, String> {
    let dirty_paths: Vec<String> = changes.iter().map(|c| c.path().to_string()).collect();
    let scan = if dirty_paths.is_empty() {
        component_incremental_scan(
            registry,
            ComponentIncrementalScanRequest {
                detect_changes: true,
                ..Default::default()
            },
        )
    } else {
        component_incremental_scan(
            registry,
            ComponentIncrementalScanRequest {
                dirty_paths,
                ..Default::default()
            },
        )
    };
    if !scan.ok {
        return Err(scan.error.unwrap_or_else(|| "scan failed".to_string()));
    }
    let instances = scan.instances.unwrap_or_default();
    let findings = scan.findings.unwrap_or_default();
    let instance = instances.into_iter().find(|i| i.id == instance_id);
    let instance_findings: Vec<ComponentFinding> = findings
        .into_iter()
        .filter(|f| f.instance_id.as_deref() == Some(instance_id))
        .collect();
    Ok(RescanResult {
        instance,
        findings: if instance_findings.is_empty() {
            None
        } else {
            Some(instance_findings)
        },
    })
}

pub fn hook_to_json(value: &Value) -> Result<serde_json::Value, String> {
    lua_to_json(value)
}

pub fn scan_single_component(
    ctx: &ComponentRunContext,
    component: &ComponentDef,
) -> Result<(Vec<ComponentInstance>, Vec<ComponentFinding>), String> {
    scan_component(ctx, component)
}
