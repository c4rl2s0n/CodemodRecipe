//! Build Luau index/graph facades for relate and verify.

use mlua::{Function, Lua, Table};

use codemod_recipe_lua::eval_return_table;
use codemod_recipe_lua::json_lua::json_to_lua;

use crate::relations::types::{ComponentEdge, FlatInstance};

const GRAPH_FACADE_LUA: &str = include_str!("graph_facade.luau");

pub fn build_index_facade(lua: &Lua, flat: &[FlatInstance]) -> Result<Table, String> {
    let module =
        eval_return_table(lua, GRAPH_FACADE_LUA, "graph_facade").map_err(|e| e.to_string())?;
    let build: Function = module
        .get("build_index")
        .map_err(|e| format!("graph facade build_index missing: {e}"))?;
    let flat_json: Vec<serde_json::Value> = flat.iter().map(|f| f.to_json()).collect();
    let data = json_to_lua(lua, &serde_json::Value::Array(flat_json))?;
    build
        .call::<Table>(data)
        .map_err(|e| format!("graph facade build_index failed: {e}"))
}

/// Build the `graph` facade (verify phase, operations) inside the given sandbox.
pub fn build_graph_facade(
    lua: &Lua,
    flat: &[FlatInstance],
    edges: &[ComponentEdge],
) -> Result<Table, String> {
    let flat_json: Vec<serde_json::Value> = flat.iter().map(|f| f.to_json()).collect();
    let edges_json = serde_json::to_value(edges).map_err(|e| e.to_string())?;
    build_graph_facade_from_json(lua, &serde_json::Value::Array(flat_json), &edges_json)
}

/// Build the graph facade from already-serialized flat instances + edges
/// (used by delegate fan-out that only has the HostInner JSON snapshot).
pub fn build_graph_facade_from_json(
    lua: &Lua,
    flat_json: &serde_json::Value,
    edges_json: &serde_json::Value,
) -> Result<Table, String> {
    let module =
        eval_return_table(lua, GRAPH_FACADE_LUA, "graph_facade").map_err(|e| e.to_string())?;
    let build: Function = module
        .get("build_graph")
        .map_err(|e| format!("graph facade build_graph missing: {e}"))?;
    let data = json_to_lua(lua, flat_json)?;
    let edges = json_to_lua(lua, edges_json)?;
    build
        .call::<Table>((data, edges))
        .map_err(|e| format!("graph facade build_graph failed: {e}"))
}

