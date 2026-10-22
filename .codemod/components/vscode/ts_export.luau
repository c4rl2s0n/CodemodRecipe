--- One top-level export in a TypeScript module.

return {
	id = "repo.ts_export",
	description = "Single exported binding / type / statement in a TS module",
	parse = function(_c, context)
		local hints = context.hints or {}
		local source = context.source
		if not source or not source.path or not source.span then
			return nil
		end
		-- Include path so anonymous export#N@offset ids do not collide across modules.
		return {
			id = source.path .. ":" .. (hints.name or "export") .. "@" .. tostring(source.span.from),
			componentId = "repo.ts_export",
			source = source,
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
			kind = "ts-export",
			title = instance.ast.name or instance.ast.id,
			id = instance.ast.id,
		}
	end,
}
