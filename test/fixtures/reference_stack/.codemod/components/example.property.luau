return {
	id = "example.property",
	description = "Entity field parsed from parent context",
	discover = function(_c)
		return {}
	end,
	identify = function(_c, locator)
		return locator.field ~= nil
	end,
	parse = function(_c, locator)
		return {
			name = locator.field,
			type = "String",
		}
	end,
	verify = function(_c, _instance)
		return {}
	end,
}
