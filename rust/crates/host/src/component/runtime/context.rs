//! Component Lua run context and host construction.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;

use codemod_recipe_engine::{LanguageRegistry, RegistryConfig};
use codemod_recipe_lua::json_lua::json_to_lua;
use codemod_recipe_lua::model::ComponentDef;
use codemod_recipe_lua::{eval_return_table, require_chunk_name};
use mlua::{Function, Lua, Table, Value};

use crate::component::types::ComponentInstance;
use crate::engine::working_tree::WorkingTree;
use crate::host_api::context::{HostInner, RunKind, SharedHost};
use crate::lua_runtime::build_ctx;
use crate::recipe::registry::RecipeRegistry;
use crate::sandbox::path_sandbox::PathSandbox;

pub struct ComponentRunContext {
    pub kind: RunKind,
    pub maps: BTreeMap<String, BTreeMap<String, String>>,
    pub vars: BTreeMap<String, BTreeMap<String, String>>,
    pub queries: BTreeMap<String, String>,
    pub recipes: BTreeMap<String, codemod_recipe_lua::model::Recipe>,
    pub components: BTreeMap<String, ComponentDef>,
    /// Stack relation-kind registry (layout = edge|member). Empty when no stack.
    pub relations: BTreeMap<String, codemod_recipe_lua::RelationKindDef>,
    pub workspace_root: PathBuf,
    pub codemod_root: PathBuf,
    pub language_config: RegistryConfig,
}

pub(crate) fn component_chunk_name(component: &ComponentDef) -> String {
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
            relations: registry
                .stack()
                .map(|s| s.relations.clone())
                .unwrap_or_default(),
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
pub(crate) fn facade_node(facade: &Table, instance: &serde_json::Value) -> Option<Table> {
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
        graph_flat_json: None,
        graph_edges_json: None,
    })))
}

pub(crate) fn set_host_graph_snapshot(
    host: &SharedHost,
    instances: &[ComponentInstance],
    edges: &[crate::component::relations::ComponentEdge],
) -> Result<(), String> {
    let flat = crate::component::relations::flatten_instances(instances);
    let flat_json: Vec<serde_json::Value> = flat.iter().map(|f| f.to_json()).collect();
    let edges_json = serde_json::to_value(edges).map_err(|e| e.to_string())?;
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    inner.graph_flat_json = Some(serde_json::Value::Array(flat_json));
    inner.graph_edges_json = Some(edges_json);
    Ok(())
}

