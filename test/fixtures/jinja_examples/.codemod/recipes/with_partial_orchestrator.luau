return {
	id = "jinja.bind.orchestrator_partial",
	name = "with_partial_orchestrator",
	description = "Partial with: remaps className from featureName; suffix falls through from parent.",
	args = {
		{ name = "featureName", required = true, inputKind = "symbol" },
		{ name = "suffix", required = true, inputKind = "text" },
	},
	apply = function(c)
		c:run("jinja.bind.child", {
			className = c.args.featureName,
			suffix = c.args.suffix,
		})
	end,
}
