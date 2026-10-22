return {
	id = "jinja.bind.orchestrator",
	name = "with_bind_orchestrator",
	description = "Forwards featureName → className and hardcodes suffix via recipe with bindings.",
	args = {
		{ name = "featureName", required = true, inputKind = "symbol" },
	},
	apply = function(c, args)
		c:run("jinja.bind.child", {
			className = args.featureName,
			suffix = "Widget",
		})
	end,
}
