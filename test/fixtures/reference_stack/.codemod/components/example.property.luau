return {
	id = "example.property",
	description = "Entity field parsed from parent context",
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
