--- Resolve which stack-root instance a product test should `tests`→.
--- After domain path opt-outs, production files may be owned by domain composers
--- instead of repo.rust_mod_file.

local M = {}

--- Path-keyed roots (instance id == componentId .. "/" .. path).
local PATH_KEYED_PREFIXES = {
	"repo.rust_mod_file/",
	"repo.support_crate_surface/",
	"repo.engine_surface/",
	"repo.lua_surface/",
}

--- Logical units whose ast.source.path (or extraPaths) matches the production file.
local PATH_SCAN_COMPONENTS = {
	"repo.component_pipeline",
	"repo.host_module",
	"repo.engine_edit",
	"repo.working_tree",
	"repo.recipe_registry",
	"repo.domain_type",
	"repo.host_api_handlers",
	"repo.host_protocol",
	"repo.mcp_tool",
	"repo.protocol_key",
	"repo.host_api_method",
	"repo.lua_load_module",
	"repo.lua_host_api_specs",
}

--- Multi-file units: production path → fixed instance id (when source.path is a sibling).
local PATH_ALIASES = {
	["rust/crates/engine/src/ops.rs"] = "repo.engine_edit/engine-edit",
	["rust/crates/host/src/protocol/mcp_inventory.rs"] = "repo.mcp_tool/mcp-surface",
}

local function instancePath(inst)
	if not inst then
		return nil
	end
	local ast = inst.ast or inst
	if ast.source and ast.source.path then
		return ast.source.path
	end
	return ast.path
end

local function instanceExtraPaths(inst)
	local ast = inst.ast or inst
	local out = {}
	if type(ast.opsPath) == "string" then
		out[#out + 1] = ast.opsPath
	end
	if type(ast.extraPaths) == "table" then
		for _, p in ipairs(ast.extraPaths) do
			out[#out + 1] = p
		end
	end
	return out
end

local function fullId(cid, inst)
	local id = inst.id or (inst.ast and inst.ast.id)
	if not id then
		return nil
	end
	if id:find("^" .. cid .. "/", 1) then
		return id
	end
	return cid .. "/" .. id
end

--- Return full instance id to edge to, or nil.
function M.testsTarget(index, path)
	if type(path) ~= "string" or path == "" then
		return nil
	end
	local alias = PATH_ALIASES[path]
	if alias and index:byId(alias) then
		return alias
	end
	for _, prefix in ipairs(PATH_KEYED_PREFIXES) do
		local to = prefix .. path
		if index:byId(to) then
			return to
		end
	end
	for _, cid in ipairs(PATH_SCAN_COMPONENTS) do
		local ok, list = pcall(function()
			return index:byComponent(cid)
		end)
		if ok and list then
			for _, inst in ipairs(list) do
				if instancePath(inst) == path then
					return fullId(cid, inst)
				end
				for _, extra in ipairs(instanceExtraPaths(inst)) do
					if extra == path then
						return fullId(cid, inst)
					end
				end
			end
		end
	end
	return nil
end

return M
