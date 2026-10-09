--- Logical unit: recipe/component/stack catalog (RecipeRegistry).

local REGISTRY = "rust/crates/host/src/recipe/registry.rs"

return {
	id = "repo.recipe_registry",
	description = "RecipeRegistry: load/reload catalog of recipes, components, stack",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles(REGISTRY)) do
			out[#out + 1] = { path = path }
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local typeSpan, hasType = nil, false
		local names = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.type_items",
			capture = "typeName",
		})
		local items = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.type_items",
			capture = "typeItem",
		})
		for i, n in ipairs(names) do
			if n.text == "RecipeRegistry" then
				hasType = true
				typeSpan = items[i] and items[i].span
			end
		end
		local children = {}
		if typeSpan then
			children[#children + 1] = {
				id = "RecipeRegistry",
				componentId = "repo.host_module_item",
				source = { path = path, span = typeSpan },
				kind = "type",
				name = "RecipeRegistry",
				role = "type",
			}
		end
		-- Pub methods on the registry file.
		pcall(function()
			local fnNames = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.pub_functions",
				capture = "fnName",
			})
			local fns = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.pub_functions",
				capture = "fn",
			})
			for i = 1, math.min(#fnNames, #fns) do
				children[#children + 1] = {
					id = "fn:" .. fnNames[i].text,
					componentId = "repo.host_module_item",
					source = { path = path, span = fns[i].span },
					kind = "fn",
					name = fnNames[i].text,
					role = "fn",
				}
			end
		end)
		return {
			id = "recipe-registry",
			componentId = "repo.recipe_registry",
			source = { path = path },
			hasRecipeRegistry = hasType,
			children = children,
		}
	end,
	relate = function(_c, instance, index)
		local edges = {
			{ kind = "lives_in", to = "repo.zone/rust.runtime" },
		}
		local apis = index:byComponent("repo.host_api_method")
		if apis and #apis > 0 then
			edges[#edges + 1] = {
				kind = "feeds",
				to = apis[1].id,
			}
		end
		return edges
	end,
	verify = function(_c, instance)
		if instance.ast.hasRecipeRegistry then
			return {}
		end
		return {
			{
				severity = "error",
				code = "recipe_registry.missing_type",
				message = "RecipeRegistry struct not found in " .. REGISTRY,
				instanceId = instance.id,
			},
		}
	end,
	toSummary = function(_c, instance)
		return {
			kind = "recipe-registry",
			title = "Recipe registry",
			id = instance.ast.id,
			facts = {},
		}
	end,
}
