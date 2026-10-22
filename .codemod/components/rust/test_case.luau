--- One #[test] function or helper fn inside a rust_test_module / rust_test file.

return {
	id = "repo.rust_test_case",
	description = "Single Rust test function or helper inside a test module/file",
	parse = function(_c, context)
		local hints = context.hints or {}
		local source = context.source
		if not source or not source.path or not source.span then
			return nil
		end
		return {
			id = source.path .. ":" .. (hints.kind or "fn") .. ":" .. (hints.name or "?") .. "@" .. tostring(
				source.span.from
			),
			componentId = "repo.rust_test_case",
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
			kind = "rust-test-case",
			title = instance.ast.name or instance.ast.id,
			id = instance.ast.id,
			facts = { instance.ast.kind or "" },
		}
	end,
}
