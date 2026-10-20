return {
	id = "repo.vscode_command_entry",
	description = "One package.json contributes.commands entry (child of repo.vscode_command)",
	parse = function(_c, context)
		local hints = context.hints or {}
		local source = context.source
		if not source or not source.path or not hints.commandId then
			return nil
		end
		return {
			id = hints.commandId,
			componentId = "repo.vscode_command_entry",
			source = source,
			commandId = hints.commandId,
			title = hints.title,
			-- bridge methods invoked from the registration handler (resolved by the parent)
			bridgeCalls = hints.bridgeCalls or {},
			handlerAnalyzed = hints.handlerAnalyzed or false,
			handlerEmpty = hints.handlerEmpty or false,
			touchesWebview = hints.touchesWebview or false,
			registrationFile = hints.registrationFile,
		}
	end,
	relate = function(_c, instance, _index)
		local ast = instance.ast
		local edges = {}
		for _, method in ipairs(ast.bridgeCalls or {}) do
			-- ref form: unresolved targets surface as relation.unresolved drift findings
			edges[#edges + 1] = {
				kind = "invokes",
				to = { componentId = "repo.host_bridge_method", key = method },
				source = ast.source,
			}
		end
		-- UI-only / extension-local commands: honest presents edge (not fake invokes).
		if #edges == 0 and ast.handlerAnalyzed then
			local zone = "vscode.extension"
			local reg = ast.registrationFile or ""
			if ast.touchesWebview or reg:find("/webview/", 1, true) then
				zone = "vscode.webview"
			end
			edges[#edges + 1] = {
				kind = "presents",
				to = "repo.zone/" .. zone,
				source = ast.source,
			}
		end
		if #edges == 0 then
			return nil
		end
		return edges
	end,
	toSummary = function(_c, instance)
		local ast = instance.ast
		return {
			kind = "vscode-command",
			title = ast.commandId,
			id = ast.id,
			path = ast.source and ast.source.path or nil,
			facts = ast.title and { ast.title } or {},
		}
	end,
}
