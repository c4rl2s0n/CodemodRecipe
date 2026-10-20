--- Pub fn or type inside a host module directory.

return {
	id = "repo.host_module_item",
	description = "Public function or type item under a host module",
	parse = function(_c, context)
		local hints = context.hints or {}
		local source = context.source
		if not source or not source.path or not source.span then
			return nil
		end
		return {
			id = source.path .. ":" .. (hints.name or "?") .. "@" .. tostring(source.span.from),
			componentId = "repo.host_module_item",
			source = source,
			kind = hints.kind,
			name = hints.name,
		}
	end,
	relate = function(_c, instance)
		if instance.parentId then
			return { { kind = "lives_in", to = instance.parentId } }
		end
		return {}
	end,
	toSummary = function(_c, instance)
		return {
			kind = "host-module-item",
			title = instance.ast.name,
			id = instance.ast.id,
			facts = { instance.ast.kind or "" },
		}
	end,
}
