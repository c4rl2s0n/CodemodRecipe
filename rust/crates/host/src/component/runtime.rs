use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;

use codemod_recipe_engine::{LanguageRegistry, RegistryConfig};
use codemod_recipe_lua::json_lua::{json_to_lua, lua_to_json};
use codemod_recipe_lua::model::ComponentDef;
use codemod_recipe_lua::{eval_return_table, new_sandbox_with_require, require_chunk_name};
use mlua::{Function, Lua, Table, Value};

use crate::component::scan::{
    component_incremental_scan, component_scan, ComponentIncrementalScanRequest,
};
use crate::component::types::{ComponentFinding, ComponentInstance, ComponentInvokeResponse};
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

fn component_chunk_name(component: &ComponentDef) -> String {
    component
        .path
        .as_ref()
        .map(|p| require_chunk_name(p))
        .unwrap_or_else(|| component.id.clone())
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

pub(crate) fn invoke_component_hook(
    lua: &Lua,
    _ctx: &ComponentRunContext,
    component: &ComponentDef,
    hook: &str,
    host: SharedHost,
    arg: Option<serde_json::Value>,
) -> Result<Value, String> {
    let ctx_table = build_ctx(lua, host)?;
    let name = component_chunk_name(component);
    let module = eval_return_table(lua, &component.source, &name).map_err(|e| e.to_string())?;
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

/// Calls an optional component hook as `hook(c, instance, facade)`; returns `None`
/// when the hook is absent. Used by the post-scan phases (`relate` / `verify`)
/// and by facade-aware dry-runs.
///
/// When the facade already holds a node table for `instance.id`, that table is
/// passed as the instance arg instead of a fresh conversion — node identity is
/// unified, so node-centric traversal (`inst:outgoing` etc., graph facade only)
/// works on the hook's own instance.
pub(crate) fn invoke_component_hook_with_facade(
    lua: &Lua,
    component: &ComponentDef,
    hook: &str,
    host: SharedHost,
    instance: serde_json::Value,
    facade: &Table,
) -> Result<Option<Value>, String> {
    let name = component_chunk_name(component);
    let module = eval_return_table(lua, &component.source, &name).map_err(|e| e.to_string())?;
    let Some(func) = module
        .get::<Option<Function>>(hook)
        .map_err(|e| e.to_string())?
    else {
        return Ok(None);
    };
    let ctx_table = build_ctx(lua, host)?;
    let instance_lua = match facade_node(facade, &instance) {
        Some(table) => Value::Table(table),
        None => json_to_lua(lua, &instance)?,
    };
    func.call::<Value>((ctx_table, instance_lua, facade.clone()))
        .map(Some)
        .map_err(|e| format!("component {hook} failed: {e}"))
}

/// Look up the facade's own node table for the instance's `id` (`byId` returns the
/// shared, traversal-wrapped node). `None` when the id is absent or not a table.
fn facade_node(facade: &Table, instance: &serde_json::Value) -> Option<Table> {
    let id = instance.get("id").and_then(|v| v.as_str())?;
    let by_id: Function = facade.get("byId").ok()?;
    match by_id.call::<Value>((facade.clone(), id.to_string())).ok()? {
        Value::Table(table) => Some(table),
        _ => None,
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
    let name = component_chunk_name(component);
    let module = eval_return_table(lua, &component.source, &name).map_err(|e| e.to_string())?;
    if module
        .get::<Option<Function>>(hook)
        .map_err(|e| e.to_string())?
        .is_none()
    {
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
        ast_cache: crate::query::ast_cache::AstParseCache::default(),
    })))
}

pub fn invoke_operation(
    registry: &mut RecipeRegistry,
    request: ComponentInvokeRequest,
) -> ComponentInvokeResponse {
    if registry.stack().is_none() {
        return ComponentInvokeResponse::fail(
            "stack manifest is required for component operations",
        );
    }
    let ctx = ComponentRunContext::from_registry(registry, request.kind);
    let scan = crate::component::scan::component_scan(registry);
    if !scan.ok {
        return ComponentInvokeResponse::fail(
            scan.error
                .unwrap_or_else(|| "component scan failed".to_string()),
        );
    }
    let instances = scan.instances.unwrap_or_default();
    let edges = scan.edges.clone().unwrap_or_default();
    let Some(target) = instances.iter().find(|i| i.id == request.instance_id) else {
        return ComponentInvokeResponse::fail(format!(
            "instance not found: {}",
            request.instance_id
        ));
    };
    let component_id = target.component_id.clone();
    let Some(component) = registry.get_component(&component_id).cloned() else {
        return ComponentInvokeResponse::fail(format!("component not found: {component_id}"));
    };
    if !component
        .operations
        .iter()
        .any(|op| op.name == request.operation)
    {
        return ComponentInvokeResponse::fail(format!(
            "unknown operation {} on component {}",
            request.operation, component_id
        ));
    }
    let host = match make_host(&ctx, request.args.clone()) {
        Ok(h) => h,
        Err(message) => return ComponentInvokeResponse::fail(message),
    };
    let lua = match new_sandbox_with_require(&ctx.codemod_root) {
        Ok(l) => l,
        Err(e) => return ComponentInvokeResponse::fail(e.to_string()),
    };
    let result = (|| -> Result<ComponentInvokeResponse, String> {
        let ctx_table = build_ctx(&lua, Rc::clone(&host))?;
        let name = component_chunk_name(&component);
        let module =
            eval_return_table(&lua, &component.source, &name).map_err(|e| e.to_string())?;
        let ops: Table = module
            .get("operations")
            .map_err(|e| format!("operations table missing: {e}"))?;
        let op_tbl: Table = ops
            .get(request.operation.as_str())
            .map_err(|_| format!("operation {} not found", request.operation))?;
        let apply: Function = op_tbl
            .get("apply")
            .map_err(|e| format!("operation apply missing: {e}"))?;
        let args_tbl = args_table_from_map(&lua, &request.args)?;
        // Operations receive the pre-operation graph as 4th arg (guards, navigation).
        let flat = crate::component::relations::flatten_instances(&instances);
        let graph = crate::component::relations::build_graph_facade(&lua, &flat, &edges)?;
        let instance_json = serde_json::json!({
            "id": target.id,
            "componentId": target.component_id,
            "locator": target.locator,
            "ast": target.ast,
        });
        // Pass the facade's own node table so `target:outgoing(...)` etc. work.
        let instance_tbl = match facade_node(&graph, &instance_json) {
            Some(table) => Value::Table(table),
            None => json_to_lua(&lua, &instance_json)?,
        };
        apply
            .call::<()>((ctx_table, args_tbl, instance_tbl, graph))
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

        let rescanned = component_scan_after_operation(registry, &target.id, &changes)?;
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
            result: None,
            instance: rescanned.instance,
            findings: rescanned.findings,
            files,
            transcript: Some(transcript),
            applied,
        })
    })();
    match result {
        Ok(resp) => resp,
        Err(message) => {
            let transcript = host
                .lock()
                .ok()
                .map(|mut inner| std::mem::take(&mut inner.transcript))
                .filter(|t| !t.is_empty());
            let mut resp = ComponentInvokeResponse::fail(message);
            resp.transcript = transcript;
            resp
        }
    }
}

pub fn invoke_scaffold(
    registry: &mut RecipeRegistry,
    request: ComponentScaffoldRequest,
) -> ComponentInvokeResponse {
    if registry.stack().is_none() {
        return ComponentInvokeResponse::fail("stack manifest is required for component scaffold");
    }
    let Some(component) = registry.get_component(&request.component_id).cloned() else {
        return ComponentInvokeResponse::fail(format!(
            "component not found: {}",
            request.component_id
        ));
    };
    if !component.has_scaffold {
        return ComponentInvokeResponse::fail(format!(
            "component {} has no scaffold entry",
            request.component_id
        ));
    }

    // Ensure cache exists so post-scaffold detectChanges can glob-diff new roots,
    // and so scaffold.apply receives the pre-scaffold graph as 3rd arg.
    let scan = component_scan(registry);
    let scan_instances = scan.instances.unwrap_or_default();
    let scan_edges = scan.edges.unwrap_or_default();

    let ctx = ComponentRunContext::from_registry(registry, request.kind);
    let host = match make_host(&ctx, request.args.clone()) {
        Ok(h) => h,
        Err(message) => return ComponentInvokeResponse::fail(message),
    };
    let lua = match new_sandbox_with_require(&ctx.codemod_root) {
        Ok(l) => l,
        Err(e) => return ComponentInvokeResponse::fail(e.to_string()),
    };
    let result = (|| -> Result<ComponentInvokeResponse, String> {
        let ctx_table = build_ctx(&lua, Rc::clone(&host))?;
        let name = component_chunk_name(&component);
        let module =
            eval_return_table(&lua, &component.source, &name).map_err(|e| e.to_string())?;
        let scaffold: Table = module
            .get("scaffold")
            .map_err(|e| format!("scaffold table missing: {e}"))?;
        let apply: Function = scaffold
            .get("apply")
            .map_err(|e| format!("scaffold apply missing: {e}"))?;
        let args_tbl = args_table_from_map(&lua, &request.args)?;
        // scaffold.apply(c, args, graph): the graph is the pre-scaffold snapshot —
        // same semantics as operations (judges the request, not mid-apply edits).
        let flat = crate::component::relations::flatten_instances(&scan_instances);
        let graph = crate::component::relations::build_graph_facade(&lua, &flat, &scan_edges)?;
        apply
            .call::<()>((ctx_table, args_tbl, graph))
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
            result: None,
            instance: None,
            findings: scan.findings,
            files,
            transcript: Some(transcript),
            applied,
        })
    })();
    match result {
        Ok(resp) => resp,
        Err(message) => {
            let transcript = host
                .lock()
                .ok()
                .map(|mut inner| std::mem::take(&mut inner.transcript))
                .filter(|t| !t.is_empty());
            let mut resp = ComponentInvokeResponse::fail(message);
            resp.transcript = transcript;
            resp
        }
    }
}

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
            applied: None,
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
                let mut resp = ComponentInvokeResponse::fail(message);
                resp.transcript = take_transcript(&host);
                return resp;
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
                    let mut resp = ComponentInvokeResponse::fail(e);
                    resp.transcript = take_transcript(&host);
                    return resp;
                }
            };
            let (applied, files, transcript) = match finalize_host(&host, request.kind) {
                Ok(parts) => parts,
                Err(e) => {
                    let mut resp = ComponentInvokeResponse::fail(e);
                    resp.transcript = take_transcript(&host);
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
                files,
                transcript: Some(transcript),
                applied,
            }
        }
        Err(message) => {
            let mut resp = ComponentInvokeResponse::fail(message);
            resp.transcript = take_transcript(&host);
            resp
        }
    }
}

fn take_transcript(host: &SharedHost) -> Option<Vec<codemod_recipe_lua::TranscriptEvent>> {
    host.lock()
        .ok()
        .map(|mut inner| std::mem::take(&mut inner.transcript))
        .filter(|t| !t.is_empty())
}

type FinalizeParts = (
    Option<Vec<String>>,
    Option<Vec<crate::protocol::FilePreview>>,
    Vec<codemod_recipe_lua::TranscriptEvent>,
);

fn finalize_host(host: &SharedHost, kind: RunKind) -> Result<FinalizeParts, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    let applied = if kind == RunKind::Apply {
        Some(inner.flush_to_disk()?)
    } else {
        None
    };
    let changes = std::mem::replace(&mut inner.tree, WorkingTree::new()).finalize();
    let transcript = std::mem::take(&mut inner.transcript);
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
    Ok((applied, files, transcript))
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
