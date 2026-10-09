return {
	id = "conditional_create",
	name = "conditional_create",
	description = "Example file-backed Jinja template with {% if %} on a bool arg.",
	args = {
		{ name = "className", required = true, inputKind = "symbol" },
		{ name = "includeTests", required = false, defaultsTo = "false", inputKind = "text" },
	},
	apply = function(c, args)
		c:create({
			path = c:template({ inline = "lib/{{ className | snake_case }}/{{ className | snake_case }}_widget.dart" }),
			content = c:template({ path = "templates/conditional_widget.dart.template" }),
			ifExists = "fail",
		})
	end,
}
