//! Load component modules and invoke parse/verify/operation/scaffold hooks.

use std::rc::Rc;

use codemod_recipe_lua::{eval_return_table, require_chunk_name};
use mlua::{Function, Lua, Table, Value};

use crate::host_api::context::SharedHost;
use crate::lua_runtime::{args_table_from_map, build_ctx};

pub(super) fn load_module(
    host: &SharedHost,
    lua: &Lua,
    component: &codemod_recipe_lua::model::ComponentDef,
) -> Result<Table, String> {
    let _ = host; // keep parity with cycle-guarded callers
    let name = component
        .path
        .as_ref()
        .map(|p| require_chunk_name(p))
        .unwrap_or_else(|| component.id.clone());
    eval_return_table(lua, &component.source, &name).map_err(|e| e.to_string())
}

pub(super) fn host_graph_facade(host: &SharedHost, lua: &Lua) -> Result<Table, String> {
    let (flat, edges) = crate::component::entry::with_component_host(host, |h| {
        Ok((h.graph_flat_json().cloned(), h.graph_edges_json().cloned()))
    })?;
    let flat = flat.ok_or_else(|| {
        "graph snapshot unavailable — call operation/scaffold only from a stack or component operation session".to_string()
    })?;
    let edges = edges.ok_or_else(|| "graph edges snapshot unavailable".to_string())?;
    crate::component::relations::build_graph_facade_from_json(lua, &flat, &edges)
}

pub(super) fn call_child_hook(
    host: &SharedHost,
    lua: &Lua,
    component: &codemod_recipe_lua::model::ComponentDef,
    hook: &str,
    arg: Option<Value>,
) -> Result<Value, String> {
    let ctx_table = build_ctx(lua, Rc::clone(host))?;
    let module = load_module(host, lua, component)?;
    let func: Function = module
        .get(hook)
        .map_err(|_| format!("component {} has no {hook} hook", component.id))?;
    match arg {
        Some(value) => func
            .call::<Value>((ctx_table, value))
            .map_err(|e| format!("component {hook} failed: {e}")),
        None => func
            .call::<Value>(ctx_table)
            .map_err(|e| format!("component {hook} failed: {e}")),
    }
}

pub(super) fn call_child_operation(
    host: &SharedHost,
    lua: &Lua,
    component: &codemod_recipe_lua::model::ComponentDef,
    operation: &str,
    args: Option<Value>,
) -> Result<Value, String> {
    if !component.operations.iter().any(|op| op.name == operation) {
        return Err(format!(
            "unknown operation {operation} on component {}",
            component.id
        ));
    }
    let ctx_table = build_ctx(lua, Rc::clone(host))?;
    let module = load_module(host, lua, component)?;
    let ops: Table = module
        .get("operations")
        .map_err(|e| format!("operations table missing: {e}"))?;
    let op_tbl: Table = ops
        .get(operation)
        .map_err(|_| format!("operation {operation} not found"))?;
    let apply: Function = op_tbl
        .get("apply")
        .map_err(|e| format!("operation apply missing: {e}"))?;
    let args_tbl = match args {
        Some(Value::Table(t)) => t,
        Some(other) => {
            return Err(format!(
                "operation args must be a table, got {}",
                other.type_name()
            ));
        }
        None => {
            let empty = std::collections::BTreeMap::<String, serde_json::Value>::new();
            args_table_from_map(lua, &empty)?
        }
    };
    let graph = host_graph_facade(host, lua)?;
    // Instance-less fan-out: pass nil for target (create-style ops ignore it).
    apply
        .call::<()>((ctx_table, args_tbl, Value::Nil, graph))
        .map_err(|e| format!("operation {operation} failed: {e}"))?;
    Ok(Value::Nil)
}

pub(super) fn call_child_scaffold(
    host: &SharedHost,
    lua: &Lua,
    component: &codemod_recipe_lua::model::ComponentDef,
    args: Option<Value>,
) -> Result<Value, String> {
    if !component.has_scaffold {
        return Err(format!("component {} has no scaffold entry", component.id));
    }
    let ctx_table = build_ctx(lua, Rc::clone(host))?;
    let module = load_module(host, lua, component)?;
    let scaffold: Table = module
        .get("scaffold")
        .map_err(|e| format!("scaffold table missing: {e}"))?;
    let apply: Function = scaffold
        .get("apply")
        .map_err(|e| format!("scaffold apply missing: {e}"))?;
    let args_tbl = match args {
        Some(Value::Table(t)) => t,
        Some(other) => {
            return Err(format!(
                "scaffold args must be a table, got {}",
                other.type_name()
            ));
        }
        None => {
            let empty = std::collections::BTreeMap::<String, serde_json::Value>::new();
            args_table_from_map(lua, &empty)?
        }
    };
    let graph = host_graph_facade(host, lua)?;
    apply
        .call::<()>((ctx_table, args_tbl, graph))
        .map_err(|e| format!("scaffold apply failed: {e}"))?;
    Ok(Value::Nil)
}
