return {
	id = "jinja.step_if.child",
	name = "step_if_child",
	description = "Atomic create used by the step-if orchestrator fixture.",
	args = {
		{ name = "className", required = true },
	},
	apply = function(c)
		c:create({
			path = c:template({ inline = "lib/generated/{{ className | snake_case }}_extra.dart" }),
			content = c:template({ inline = "class {{ className | pascal_case }}Extra {}\n" }),
		})
	end,
}
