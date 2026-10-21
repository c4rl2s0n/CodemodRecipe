--- Shared zone catalog for stack.zones / repo.zone identify (SSOT).
--- Placement (D-018 / D-025): architecture zones own real modules; structural = barrels only.
--- Domain zones replace flat rust.runtime for host internals.

local M = {
	zones = {
		{ id = "structural.modules", role = "structural", title = "Module barrels" },
		{ id = "command.adapters", role = "architecture", title = "Command adapters" },
		{ id = "domain.recipe", role = "architecture", title = "Recipe domain" },
		{ id = "domain.edit_session", role = "architecture", title = "Edit session" },
		{ id = "domain.host_api", role = "architecture", title = "Host API" },
		{ id = "domain.components", role = "architecture", title = "Components subsystem" },
		{ id = "domain.query_intel", role = "architecture", title = "Query intel" },
		{ id = "domain.types", role = "architecture", title = "Domain types" },
		{ id = "domain.sandbox", role = "architecture", title = "Path sandbox / globs" },
		{ id = "domain.config", role = "architecture", title = "Host config / bootstrap" },
		{ id = "domain.template", role = "architecture", title = "MiniJinja templates" },
		{ id = "domain.explorer", role = "architecture", title = "Explorer recipe filter" },
		{ id = "platform.engine", role = "architecture", title = "Engine platform" },
		{ id = "platform.lua", role = "architecture", title = "Lua platform" },
		{ id = "vscode.extension", role = "architecture", title = "VS Code extension" },
		{ id = "vscode.webview", role = "architecture", title = "VS Code webview" },
		{ id = "vscode.shared", role = "architecture", title = "VS Code shared" },
		{ id = "docs.surface", role = "documentation", title = "Docs surface" },
	},
	-- structural: barrels under the zone are membership-only for arch ratio.
	-- Zone *catalog instances* themselves are excluded in relation_metrics (not here).
	zoneRoles = {
		structural = { ignoreRelations = true, ignoreCoverage = false },
	},
}

--- Map a vscode_extension/src path to its architecture zone id (without repo.zone/ prefix).
function M.vscodeZoneForPath(path)
	if type(path) ~= "string" then
		return "vscode.extension"
	end
	if path:find("/webview/", 1, true) or path:find("/src/webview/", 1, true) then
		return "vscode.webview"
	end
	if path:find("/shared/", 1, true) or path:find("/src/shared/", 1, true) then
		return "vscode.shared"
	end
	return "vscode.extension"
end

--- Map a host crate src path to a domain zone id (without repo.zone/ prefix).
function M.hostZoneForPath(path)
	if type(path) ~= "string" then
		return "domain.recipe"
	end
	if path:find("/bin/codemod_mcp", 1, true)
		or path:find("/bin/codemod_host", 1, true)
		or path:find("/bin/generate_host_api", 1, true)
	then
		return "command.adapters"
	end
	if path:find("/edit_session/", 1, true) or path:find("/preview/", 1, true) then
		return "domain.edit_session"
	end
	if path:find("/host_api/", 1, true) or path:find("/lua_runtime", 1, true) then
		return "domain.host_api"
	end
	if path:find("/component/", 1, true) or path:match("testing%.rs$") then
		return "domain.components"
	end
	if path:find("/query/", 1, true) then
		return "domain.query_intel"
	end
	if path:find("/sandbox/", 1, true) then
		return "domain.sandbox"
	end
	if path:find("/config/", 1, true) then
		return "domain.config"
	end
	if path:find("/template/", 1, true) then
		return "domain.template"
	end
	if path:find("/explorer/", 1, true) then
		return "domain.explorer"
	end
	if path:find("/recipe/", 1, true) then
		return "domain.recipe"
	end
	if path:find("/protocol/", 1, true) then
		return "command.adapters"
	end
	return "domain.recipe"
end

return M
