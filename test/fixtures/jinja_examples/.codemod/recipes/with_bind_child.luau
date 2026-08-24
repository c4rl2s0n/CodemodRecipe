return {
	id = "jinja.bind.child",
	name = "with_bind_child",
	description = "Child recipe for call-site with bindings (className + suffix)",
	args = {
		{ name = "className", required = true, inputKind = "symbol" },
		{ name = "suffix", required = true, inputKind = "text" },
	},
	apply = function(c)
		c:create({
			path = c:applyTemplate({ template = "lib/generated/{{ className | snake_case }}_{{ suffix }}.dart" }),
			content = c:applyTemplate({ template = "class {{ className | pascal_case }}{{ suffix | pascal_case }} {}\n" }),
			ifExists = "fail",
		})
	end,
}
