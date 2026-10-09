--- One webview message const entry (child of repo.webview_message).

return {
	id = "repo.webview_message_entry",
	description = "Single webview message type key",
	parse = function(_c, context)
		local hints = context.hints or {}
		local source = context.source
		if not source or not source.path then
			return nil
		end
		return {
			id = hints.id or hints.name,
			componentId = "repo.webview_message_entry",
			source = source,
			direction = hints.direction,
			name = hints.name,
		}
	end,
	relate = function(_c, instance)
		-- Inventory under webview zone; dead constKey→command invokes removed (never fired).
		local edges = {
			{ kind = "lives_in", to = "repo.zone/vscode.webview" },
		}
		if instance.parentId then
			edges = { { kind = "lives_in", to = instance.parentId } }
		end
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "webview-message",
			title = instance.ast.name,
			id = instance.ast.id,
			facts = { instance.ast.direction or "" },
		}
	end,
}
