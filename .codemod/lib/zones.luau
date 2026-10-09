--- Shared zone catalog for stack.zones / repo.zone identify (SSOT).
--- Placement (D-018): architecture zones own real modules; structural = barrels only.
--- No inventory catch-all — coverage leaves nest under their parent via lives_in.

local M = {
	zones = {
		{ id = "structural.modules", role = "structural", title = "Module barrels" },
		{ id = "command.adapters", role = "architecture", title = "Command adapters" },
		{ id = "rust.runtime", role = "architecture", title = "Rust runtime" },
		{ id = "domain.types", role = "architecture", title = "Domain types" },
		{ id = "vscode.extension", role = "architecture", title = "VS Code extension" },
		{ id = "vscode.webview", role = "architecture", title = "VS Code webview" },
		{ id = "vscode.shared", role = "architecture", title = "VS Code shared" },
		{ id = "docs.surface", role = "documentation", title = "Docs surface" },
	},
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

return M
