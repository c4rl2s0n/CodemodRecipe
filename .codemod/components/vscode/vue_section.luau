--- One SFC block: script / template / style (coarse span leaf).

return {
	id = "repo.vue_section",
	description = "Vue SFC section leaf (script, template, or style block)",
	parse = function(_c, context)
		local hints = context.hints or {}
		local source = context.source
		if not source or not source.path or not source.span or not hints.kind then
			return nil
		end
		return {
			id = source.path .. ":" .. hints.kind .. "@" .. tostring(source.span.from),
			componentId = "repo.vue_section",
			source = source,
			kind = hints.kind,
			name = hints.kind,
		}
	end,
	relate = function(_c, instance)
		if instance.parentId then
			return { { kind = "lives_in", to = instance.parentId } }
		end
		return {}
	end,
	toSummary = function(_c, instance)
		local k = instance.ast.kind or "section"
		return {
			kind = "vue-section",
			title = k,
			id = instance.ast.id,
			facts = { k, instance.ast.source and instance.ast.source.path or "" },
		}
	end,
}
