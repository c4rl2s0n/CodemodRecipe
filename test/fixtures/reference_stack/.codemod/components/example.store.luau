return {
	id = "example.store",
	description = "Store files under src/stores",
	discover = function(c)
		local paths = c:glob("src/stores/*_store.dart")
		local locators = {}
		for i, path in ipairs(paths) do
			locators[i] = { path = path }
		end
		return locators
	end,
	identify = function(_c, locator)
		return locator.path ~= nil and locator.path:find("_store.dart", 1, true) ~= nil
	end,
	parse = function(c, locator)
		return {
			id = c:stem(locator.path):gsub("_store$", ""),
			name = c:stem(locator.path),
			path = locator.path,
		}
	end,
	verify = function(_c, _instance)
		return {}
	end,
}
