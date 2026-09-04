return {
	id = "flutter.state",
	description = "Mutable store classes under lib/stores",
	parse = function(c, context)
		local filePath = context.source and context.source.path or context.path
		if not filePath then
			return nil
		end

		local parent = context.parent
		local modelId = parent and parent.id or c:stem(filePath):gsub("_store$", "")
		local classHits = c:query({
			path = filePath,
			language = "dart",
			query = "flutter.model_class",
			capture = "className",
		})
		local className = (#classHits > 0 and classHits[1].text) or c:stem(filePath)

		local fields = {}
		local fieldHits = c:query({
			path = filePath,
			language = "dart",
			query = "flutter.model_field",
			capture = "fieldName",
		})
		for _, hit in ipairs(fieldHits) do
			fields[#fields + 1] = {
				name = hit.text,
				source = {
					path = filePath,
					span = { start = hit.start, ["end"] = hit["end"] },
				},
			}
		end

		return {
			id = modelId,
			name = className,
			componentId = "flutter.state",
			modelId = modelId,
			source = { path = filePath },
			fields = fields,
		}
	end,
	verify = function(_c, _instance)
		return {}
	end,
}
