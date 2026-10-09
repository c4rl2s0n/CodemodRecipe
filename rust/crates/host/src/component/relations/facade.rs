//! Luau index/graph facades for relate and verify phases.

use mlua::{Function, Lua, Table};

use codemod_recipe_lua::eval_return_table;
use codemod_recipe_lua::json_lua::json_to_lua;

use crate::component::relations::types::{ComponentEdge, FlatInstance};

/// Luau chunk building the `index` / `graph` facades from plain JSON data.
/// Kept in Luau (not mlua closures) so the query logic stays readable and hackable.
///
/// Vocabulary: instances are the graph **nodes**, relations are **edges**. The
/// parse-hook payload on a node is `inst.ast` ("node" only ever means a vertex).
const GRAPH_FACADE_LUA: &str = r#"
local M = {}

local function index_maps(flat)
	local byId, byComponent = {}, {}
	for _, inst in ipairs(flat) do
		byId[inst.id] = inst
		local list = byComponent[inst.componentId]
		if list == nil then
			list = {}
			byComponent[inst.componentId] = list
		end
		list[#list + 1] = inst
	end
	return byId, byComponent
end

local function filter_kind(list, kind)
	if kind == nil then
		return list
	end
	local out = {}
	for _, edge in ipairs(list) do
		if edge.kind == kind then
			out[#out + 1] = edge
		end
	end
	return out
end

function M.build_index(flat)
	local byId, byComponent = index_maps(flat)
	return {
		byId = function(_self, id)
			return byId[id]
		end,
		byComponent = function(_self, componentId)
			local list = byComponent[componentId]
			if list == nil then
				return {}
			end
			return list
		end,
		nodes = function(_self)
			return flat
		end,
	}
end

function M.build_graph(flat, edges)
	local byId, byComponent = index_maps(flat)
	local out, inc = {}, {}
	for _, edge in ipairs(edges) do
		local outList = out[edge.from]
		if outList == nil then
			outList = {}
			out[edge.from] = outList
		end
		outList[#outList + 1] = edge
		if edge.to ~= nil then
			local incList = inc[edge.to]
			if incList == nil then
				incList = {}
				inc[edge.to] = incList
			end
			incList[#incList + 1] = edge
		end
	end

	-- Node-centric traversal: every node table carries bound methods via metatable.
	-- Methods live on __index, so pairs()/JSON serialization stay clean.
	local nodeMT = {
		__index = function(inst, key)
			if key == "outgoing" then
				return function(_self, kind)
					return filter_kind(out[inst.id] or {}, kind)
				end
			elseif key == "incoming" then
				return function(_self, kind)
					return filter_kind(inc[inst.id] or {}, kind)
				end
			elseif key == "follow" then
				-- Other endpoint of the edge (nil for external symbol targets).
				return function(_self, edge)
					if edge.from == inst.id then
						if edge.to == nil then
							return nil
						end
						return byId[edge.to]
					end
					return byId[edge.from]
				end
			elseif key == "neighbors" then
				-- Deduped adjacent nodes across both directions (kind-filtered).
				return function(_self, kind)
					local seen, result = {}, {}
					local function add(id)
						if id ~= nil and id ~= inst.id and seen[id] == nil and byId[id] ~= nil then
							seen[id] = true
							result[#result + 1] = byId[id]
						end
					end
					for _, edge in ipairs(out[inst.id] or {}) do
						if kind == nil or edge.kind == kind then
							add(edge.to)
						end
					end
					for _, edge in ipairs(inc[inst.id] or {}) do
						if kind == nil or edge.kind == kind then
							add(edge.from)
						end
					end
					return result
				end
			end
			return nil
		end,
	}
	for _, inst in ipairs(flat) do
		setmetatable(inst, nodeMT)
	end

	return {
		byId = function(_self, id)
			return byId[id]
		end,
		byComponent = function(_self, componentId)
			local list = byComponent[componentId]
			if list == nil then
				return {}
			end
			return list
		end,
		nodes = function(_self)
			return flat
		end,
		edges = function(_self)
			return edges
		end,
		outgoing = function(_self, id, kind)
			return filter_kind(out[id] or {}, kind)
		end,
		incoming = function(_self, id, kind)
			return filter_kind(inc[id] or {}, kind)
		end,
		resolve = function(_self, edge)
			if edge.to == nil then
				return nil
			end
			return byId[edge.to]
		end,
		resolveFrom = function(_self, edge)
			return byId[edge.from]
		end,
		reachable = function(_self, fromId, kind)
			local seen = {}
			local queue = { fromId }
			while #queue > 0 do
				local current = table.remove(queue, 1)
				for _, edge in ipairs(out[current] or {}) do
					if (kind == nil or edge.kind == kind) and edge.to ~= nil and seen[edge.to] == nil then
						seen[edge.to] = true
						queue[#queue + 1] = edge.to
					end
				end
			end
			return seen
		end,
	}
end

return M
"#;

/// Build the `index` facade (relate phase) inside the given sandbox.
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

