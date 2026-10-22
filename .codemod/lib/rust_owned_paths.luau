--- Paths owned by domain/platform stack roots — structural inventories must skip.
--- Mirrors vscode ts_module domain opt-outs (D-029 identify_path_conflict).

local M = {}

local EXACT = {
	-- repo.host_module MODULES barrels + leaf modules
	["rust/crates/host/src/sandbox/mod.rs"] = true,
	["rust/crates/host/src/edit_session/preview/mod.rs"] = true,
	["rust/crates/host/src/template/mod.rs"] = true,
	["rust/crates/host/src/query/mod.rs"] = true,
	["rust/crates/host/src/explorer/mod.rs"] = true,
	["rust/crates/host/src/diagnostics/mod.rs"] = true,
	["rust/crates/host/src/config/mod.rs"] = true,
	["rust/crates/host/src/host_api/mod.rs"] = true,
	["rust/crates/host/src/recipe/mod.rs"] = true,
	["rust/crates/host/src/protocol/mod.rs"] = true,
	["rust/crates/host/src/edit_session/mod.rs"] = true,
	["rust/crates/host/src/testing.rs"] = true,
	["rust/crates/host/src/host_api/lua_runtime.rs"] = true,
	["rust/crates/host/src/host_api/registry.rs"] = true,
	-- singles
	["rust/crates/host/src/recipe/registry.rs"] = true,
	["rust/crates/host/src/edit_session/working_tree.rs"] = true,
	["rust/crates/host/src/protocol/command.rs"] = true,
	["rust/crates/host/src/protocol/keys.rs"] = true,
	["rust/crates/host/src/protocol/mcp_inventory.rs"] = true,
	["rust/crates/host/src/bin/codemod_mcp.rs"] = true,
	["rust/crates/engine/src/engine/mod.rs"] = true,
	-- repo.domain_type leaves
	["rust/crates/core/src/query_spec.rs"] = true,
	["rust/crates/core/src/apply/file_change.rs"] = true,
	["rust/crates/lua/src/host_api/specs/query.rs"] = true,
	["rust/crates/lua/src/host_api/specs/common.rs"] = true,
}

local PREFIXES = {
	"rust/crates/host/src/component/",
	"rust/crates/host/src/host_api/handlers/",
	"rust/crates/core/src/",
	"rust/crates/query_tools/src/",
	"rust/crates/lua_macros/src/",
	"rust/crates/engine/src/",
	"rust/crates/lua/src/",
}

--- True when `repo.rust_mod_file` must not identify `path`.
function M.isDomainOwned(path)
	if type(path) ~= "string" then
		return false
	end
	if EXACT[path] then
		return true
	end
	for _, prefix in ipairs(PREFIXES) do
		if path:sub(1, #prefix) == prefix then
			return true
		end
	end
	return false
end

--- domain_type leaves — support_crate_surface must yield these.
local DOMAIN_TYPE_LEAVES = {
	["rust/crates/core/src/query_spec.rs"] = true,
	["rust/crates/core/src/apply/file_change.rs"] = true,
	["rust/crates/lua/src/host_api/specs/query.rs"] = true,
	["rust/crates/lua/src/host_api/specs/common.rs"] = true,
}

function M.isDomainTypeLeaf(path)
	return DOMAIN_TYPE_LEAVES[path] == true
end

--- engine_edit owns these paths — engine_surface must yield them.
function M.isEngineEditOwned(path)
	return path == "rust/crates/engine/src/engine/mod.rs"
		or path == "rust/crates/engine/src/ops.rs"
end

return M
