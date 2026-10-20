--- One top-level (or named) JSON object field — coarse span leaf.

return {
	id = "repo.json_field",
	description = "JSON object field leaf (key + value span)",
	parse = function(_c, context)
		local hints = context.hints or {}
		local source = context.source
		if not source or not source.path or not source.span or not hints.name then
			return nil
		end
		return {
			id = source.path .. ":" .. hints.name .. "@" .. tostring(source.span.from),
			componentId = "repo.json_field",
			source = source,
			name = hints.name,
			role = hints.role or "field",
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
			kind = "json-field",
			title = instance.ast.name or instance.ast.id,
			id = instance.ast.id,
			facts = { instance.ast.role or "field" },
		}
	end,
}
