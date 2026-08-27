return {
	id = "create_repository",
	name = "create_repository",
	description = "Create a repository file from a template",
	args = {
		{ name = "className", required = true, inputKind = "symbol" },
	},
	apply = function(c)
		c:create({
			path = "lib/counter/counter_repository.dart",
			content = c:template({ path = "templates/counter_repository.dart.template" }),
			ifExists = "fail",
		})
	end,
}
