return {
	id = "flutter.model.tests",
	target = "flutter.model",
	tests = {
		parses_fields_and_methods = function(t)
			local ws = t:workspace({
				files = {
					["lib/models/recipe.dart"] = "class Recipe {\n  final String title;\n  final String subtitle;\n\n  Recipe copy() => Recipe(title: title, subtitle: subtitle);\n}\n",
				},
			})
			local node = ws:parse("flutter.model", { path = "lib/models/recipe.dart", name = "recipe" })
			t:assertEq(node.name, "Recipe")
			t:assertEq(node.id, "recipe")
			local properties, methods = 0, 0
			for _, child in ipairs(node.children) do
				if child.componentId == "flutter.property" then
					properties += 1
				elseif child.componentId == "flutter.method" then
					methods += 1
				end
			end
			t:assertEq(properties, 2)
			t:assertEq(methods, 1)
		end,

		add_store_field_operation = function(t)
			local ws = t:workspace({
				files = {
					["lib/models/recipe.dart"] = "class Recipe {\n  final String title;\n}\n",
					["lib/stores/recipe_store.dart"] = "class RecipeStore {\n  String title = '';\n}\n",
				},
			})
			local scan = ws:scan()
			t:assertEq(#scan.edges, 1)
			ws:apply("flutter.model", "addStoreField", { fieldName = "rating" }, "flutter.model/recipe")
			local store = ws:readFile("lib/stores/recipe_store.dart")
			t:assert(store:find("String rating = '';") ~= nil, "store must gain the field:\n" .. store)
		end,
	},
}
