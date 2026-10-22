return {
	id = "flutter.method",
	description = "Model method parsed from parent context",
	parse = function(_c, context)
		local filePath = context.source and context.source.path or context.path
		if not filePath or not context.hints or not context.hints.name then
			return nil
		end
		return {
			name = context.hints.name,
			componentId = "flutter.method",
			source = context.source,
		}
	end,
	verify = function(_c, _instance)
		return {}
	end,
}
