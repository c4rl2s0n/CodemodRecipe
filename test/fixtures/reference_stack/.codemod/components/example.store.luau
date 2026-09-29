return {
	id = "example.store",
	description = "Store files under src/stores",
	identify = function(c)
		local out = {}
		for _, row in ipairs(c:matchFiles("src/stores/{id}_store.dart")) do
			out[#out + 1] = { path = row.path, id = row.id }
		end
		return out
	end,
	parse = function(c, locator)
		local id = locator.id or c:filename(locator.path):gsub("_store$", "")
		return {
			id = id,
			name = c:filename(locator.path),
			path = locator.path,
		}
	end,
	relate = function(_c, instance, _index)
		return {
			kind = "stores",
			to = { componentId = "example.entity", key = instance.ast.id },
			source = { path = instance.ast.path },
		}
	end,
}
