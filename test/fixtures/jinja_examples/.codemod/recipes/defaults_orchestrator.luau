return {
	id = "jinja.defaults.orchestrator",
	name = "defaults_orchestrator",
	description = "Orchestrator that inherits child defaultsTo for verbose",
	args = {
		{ name = "label", required = true, inputKind = "symbol" },
	},
	apply = function(c)
		c:run("jinja.defaults.child")
	end,
}
