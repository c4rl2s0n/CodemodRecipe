return {
	id = "flutter.state",
	description = "Mutable store classes under lib/stores",
	identify = function(c)
		local out = {}
		for _, row in ipairs(c:matchFiles("lib/stores/{id}_store.dart")) do
			out[#out + 1] = { path = row.path, id = row.id }
		end
		return out
	end,
	parse = function(c, context)
		local filePath = context.source and context.source.path or context.path
		if not filePath then
			return nil
		end

		local parent = context.parent
		local modelId = context.id or (parent and parent.id) or c:filename(filePath):gsub("_store$", "")
		local classHits = c:query({
			path = filePath,
			language = "dart",
			query = "flutter.model_class",
			capture = "className",
		})
		local className = (#classHits > 0 and classHits[1].text) or c:filename(filePath)

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
					span = { from = hit.span.from, to = hit.span.to },
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
	relate = function(_c, instance, _index)
		local ast = instance.ast
		return {
			kind = "stores",
			to = { componentId = "flutter.model", key = ast.modelId },
			source = ast.source,
		}
	end,
}
