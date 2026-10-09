--- One #[host_method] / HostMethodSurface entry (child of repo.host_api_method).

return {
	id = "repo.host_api_method_entry",
	description = "Single Luau c: host method declaration site",
	parse = function(_c, context)
		local hints = context.hints or {}
		local source = context.source
		if not source or not source.path then
			return nil
		end
		return {
			id = hints.name,
			componentId = "repo.host_api_method_entry",
			source = source,
			name = hints.name,
		}
	end,
	relate = function(_c, instance)
		if instance.parentId then
			return { { kind = "lives_in", to = instance.parentId } }
		end
		return { { kind = "lives_in", to = "repo.host_api_method/host-api-surface" } }
	end,
	toSummary = function(_c, instance)
		return {
			kind = "host-api-method",
			title = instance.ast.name,
			id = instance.ast.id,
		}
	end,
}
