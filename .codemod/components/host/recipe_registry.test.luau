return {
	id = "repo.recipe_registry.tests",
	target = "repo.recipe_registry",
	tests = {
		identifies_registry = function(t)
			local ws = t:workspace({
				files = {
					["rust/crates/host/src/recipe/registry.rs"] = [[
pub struct RecipeRegistry {}
]],
				},
			})
			local scan = ws:scan()
			local found
			for _, inst in ipairs(scan.instances) do
				if inst.componentId == "repo.recipe_registry" then
					found = inst
				end
			end
			t:assert(found, "recipe_registry missing")
			t:assert(found.ast.hasRecipeRegistry)
		end,
	},
}
