return {
	id = "flutter-poc-stack.tests",
	tests = {
		store_relations_resolve = function(t)
			local ws = t:workspace({
				files = {
					["lib/models/recipe.dart"] = "class Recipe {\n  final String title;\n}\n",
					["lib/stores/recipe_store.dart"] = "class RecipeStore {\n  String title = '';\n}\n",
				},
			})
			local scan = ws:scan()
			t:assertEq(#scan.instances, 2)
			t:assertEq(#scan.edges, 1)
			t:assertEq(scan.edges[1].kind, "stores")
			t:assertEq(scan.edges[1].from, "flutter.state/recipe")
			t:assertEq(scan.edges[1].to, "flutter.model/recipe")
			t:assertEq(scan.kindGraph[1].from, "flutter.state")
			t:assertEq(scan.kindGraph[1].to, "flutter.model")
		end,

		scaffold_creates_linked_pair = function(t)
			local ws = t:workspace({})
			ws:scaffold("flutter.model", { name = "User" })
			t:assert(ws:readFile("lib/models/user.dart") ~= nil, "model file created")
			t:assert(ws:readFile("lib/stores/user_store.dart") ~= nil, "store file created")
			local scan = ws:scan()
			t:assertEq(#scan.instances, 2)
			t:assertEq(#scan.edges, 1)
			t:assertEq(scan.edges[1].to, "flutter.model/user")
		end,

		verify_flags_model_without_store = function(t)
			local ws = t:workspace({
				files = {
					["lib/models/recipe.dart"] = "class Recipe {\n  final String title;\n}\n",
				},
			})
			local scan = ws:scan()
			local found = false
			for _, f in ipairs(scan.findings) do
				if f.code == "model.missing_store" then
					found = true
				end
			end
			t:assert(found, "expected a model.missing_store finding")
		end,
	},
}
