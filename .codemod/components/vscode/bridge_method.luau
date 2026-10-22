return {
	id = "repo.host_bridge_method",
	description = "One hostBridge method sending HostCommand payloads (child of repo.host_bridge)",
	parse = function(_c, context)
		local hints = context.hints or {}
		if not hints.method then
			return nil
		end
		return {
			id = hints.method,
			componentId = "repo.host_bridge_method",
			source = context.source,
			-- command literals sent by this method (HostCommand serde renames)
			commands = hints.commands or {},
		}
	end,
	relate = function(_c, instance, _index)
		local ast = instance.ast
		local edges = {}
		for _, command in ipairs(ast.commands or {}) do
			-- ref form: unresolved targets surface as relation.unresolved drift findings
			edges[#edges + 1] = {
				kind = "sends",
				to = { componentId = "repo.host_command", key = command },
				source = ast.source,
			}
		end
		-- Plumbing methods (no HostCommand sends): membership under bridge root.
		if #edges == 0 then
			if instance.parentId then
				return { { kind = "lives_in", to = instance.parentId } }
			end
			return { { kind = "lives_in", to = "repo.host_bridge/host-bridge" } }
		end
		return edges
	end,
	toSummary = function(_c, instance)
		local ast = instance.ast
		return {
			kind = "host-bridge-method",
			title = ast.id,
			id = ast.id,
			path = ast.source and ast.source.path or nil,
			facts = { "sends " .. table.concat(ast.commands or {}, ", ") },
		}
	end,
}
