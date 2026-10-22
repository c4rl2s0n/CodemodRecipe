return {
	id = "jinja.create.layout",
	name = "create_with_layout",
	description = "create.templateFile with extends and include",
	args = {
		{ name = "className", required = true, inputKind = "symbol" },
	},
	apply = function(c, args)
		c:create({
			path = c:template({ inline = "lib/generated/{{ className | snake_case }}_layout.dart" }),
			content = c:template({ path = "templates/layout_widget.dart.template" }),
			ifExists = "fail",
		})
	end,
}
