--- One mod or use declaration inside a rust module file.

return {
	id = "repo.rust_mod_item",
	description = "Single mod_item or use_declaration in a Rust barrel",
	parse = function(_c, context)
		local hints = context.hints or {}
		local source = context.source
		if not source or not source.path or not source.span then
			return nil
		end
		return {
			id = (hints.kind or "item") .. ":" .. (hints.name or "?") .. "@" .. tostring(source.span.from),
			componentId = "repo.rust_mod_item",
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
			kind = "rust-mod-item",
			title = instance.ast.name or instance.ast.id,
			id = instance.ast.id,
			facts = { instance.ast.kind or "" },
		}
	end,
}
