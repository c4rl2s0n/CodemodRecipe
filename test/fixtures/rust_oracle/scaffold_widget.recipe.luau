return {
	id = "scaffold_widget",
	args = {
		{ name = "featurePath", required = true },
		{ name = "name", required = true },
	},
	apply = function(c)
		c:create({
			path = c:template({ inline = "{{featurePath}}/{{ name | snake_case }}.dart" }),
			content = c:template({ inline = "class {{ name | pascal_case }} {}\n" }),
			ifExists = "fail",
		})
	end,
}
