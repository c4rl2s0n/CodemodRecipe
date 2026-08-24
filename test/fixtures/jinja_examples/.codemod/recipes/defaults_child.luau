return {
	id = "jinja.defaults.child",
	name = "defaults_child",
	description = "Child recipe with defaultsTo on a required arg",
	args = {
		{ name = "label", required = true, inputKind = "symbol" },
		{ name = "verbose", required = true, defaultsTo = "false", inputKind = "text" },
	},
	apply = function(c)
		local verbose = c.args.verbose or "false"
		c:create({
			path = c:applyTemplate({ template = "lib/generated/{{ label | snake_case }}_defaults.dart" }),
			content = c:applyTemplate({
				template = [[
class {{ label | pascal_case }}Defaults {
  static const verbose = {{ verbose }};
}
]],
				env = { verbose = verbose },
			}),
			ifExists = "fail",
		})
	end,
}
