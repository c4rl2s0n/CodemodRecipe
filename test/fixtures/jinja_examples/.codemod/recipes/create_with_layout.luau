return {
	id = "jinja.create.layout",
	name = "create_with_layout",
	description = "create.templateFile with extends and include",
	args = {
		{ name = "className", required = true, inputKind = "symbol" },
	},
	apply = function(c)
		c:create({
			path = c:applyTemplate({ template = "lib/generated/{{ className | snake_case }}_layout.dart" }),
			content = c:renderFileTemplate({ path = "templates/layout_widget.dart.template" }),
			ifExists = "fail",
		})
	end,
}
