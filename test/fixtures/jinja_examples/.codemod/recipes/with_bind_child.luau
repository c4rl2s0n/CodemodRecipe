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
			path = c:template({ inline = "lib/generated/{{ className | snake_case }}_{{ suffix }}.dart" }),
			content = c:template({ inline = "class {{ className | pascal_case }}{{ suffix | pascal_case }} {}\n" }),
			ifExists = "fail",
		})
	end,
}
