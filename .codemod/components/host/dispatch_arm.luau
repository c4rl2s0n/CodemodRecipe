--- HostCommand dispatch match arm (child helper under protocol scan).

return {
	id = "repo.host_dispatch_arm",
	description = "One HostCommand::Variant match arm in dispatch.rs",
	parse = function(_c, context)
		local hints = context.hints or {}
		local source = context.source
		if not source or not source.path or not source.span then
			return nil
		end
		return {
			id = hints.variant,
			componentId = "repo.host_dispatch_arm",
			source = source,
			variant = hints.variant,
		}
	end,
	relate = function(_c, instance, index)
		local variant = instance.ast.variant or instance.ast.id
		local edges = {
			{ kind = "lives_in", to = "repo.zone/command.adapters" },
		}
		if not variant then
			return edges
		end
		-- Resolve Rust variant → host_command instance (id is often serde rename).
		local cmds = index:byComponent("repo.host_command")
		if cmds then
			for _, cmd in ipairs(cmds) do
				local ast = cmd.ast or {}
				if ast.variant == variant or ast.id == variant or ast.rename == variant then
					edges[#edges + 1] = {
						kind = "handles",
						to = cmd.id,
						source = instance.ast.source,
					}
					return edges
				end
			end
		end
		edges[#edges + 1] = {
			kind = "handles",
			to = { componentId = "repo.host_command", key = variant },
			source = instance.ast.source,
		}
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "dispatch-arm",
			title = instance.ast.variant,
			id = instance.ast.id,
		}
	end,
}
