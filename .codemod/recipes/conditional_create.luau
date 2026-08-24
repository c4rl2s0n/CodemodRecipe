return {
	id = "conditional_create",
	name = "conditional_create",
	description = "Example file-backed Jinja template with {% if %} on a bool arg.",
	args = {
		{ name = "className", required = true, inputKind = "symbol" },
		{ name = "includeTests", required = false, defaultsTo = "false", inputKind = "text" },
	},
	apply = function(c)
		c:create({
			path = c:applyTemplate({ template = "lib/{{ className | snake_case }}/{{ className | snake_case }}_widget.dart" }),
			content = c:renderFileTemplate({ path = "templates/conditional_widget.dart.template" }),
			ifExists = "fail",
		})
	end,
}
