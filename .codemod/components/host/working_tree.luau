--- Logical unit: host WorkingTree staged edits → FileChange finalize.

local WORKING_TREE = "rust/crates/host/src/engine/working_tree.rs"

return {
	id = "repo.working_tree",
	description = "Host working tree: staged edits finalize to FileChange",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles(WORKING_TREE)) do
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
			if n.text == "WorkingTree" then
				hasType = true
				typeSpan = items[i] and items[i].span
			end
		end
		local finalizeSpan, hasFinalize = nil, false
		local fnNames = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.function_items",
			capture = "fnName",
		})
		local fns = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.function_items",
			capture = "fn",
		})
		for i, n in ipairs(fnNames) do
			if n.text == "finalize" then
				hasFinalize = true
				finalizeSpan = fns[i] and fns[i].span
			end
		end
		local children = {}
		if typeSpan then
			children[#children + 1] = {
				id = "WorkingTree",
				componentId = "repo.host_module_item",
				source = { path = path, span = typeSpan },
				kind = "type",
				name = "WorkingTree",
				role = "type",
			}
		end
		if finalizeSpan then
			children[#children + 1] = {
				id = "finalize",
				componentId = "repo.host_module_item",
				source = { path = path, span = finalizeSpan },
				kind = "fn",
				name = "finalize",
				role = "fn",
			}
		end
		return {
			id = "working-tree",
			componentId = "repo.working_tree",
			source = { path = path },
			hasWorkingTree = hasType,
			hasFinalize = hasFinalize,
			children = children,
		}
	end,
	relate = function(_c, _instance, _index)
		return {
			{ kind = "lives_in", to = "repo.zone/rust.runtime" },
			{
				kind = "finalizes",
				to = { symbol = "codemod_recipe_core::apply::file_change::FileChange" },
			},
		}
	end,
	verify = function(_c, instance)
		local findings = {}
		if not instance.ast.hasWorkingTree then
			findings[#findings + 1] = {
				severity = "error",
				code = "working_tree.missing_type",
				message = "WorkingTree struct not found in " .. WORKING_TREE,
				instanceId = instance.id,
			}
		end
		if not instance.ast.hasFinalize then
			findings[#findings + 1] = {
				severity = "warning",
				code = "working_tree.missing_finalize",
				message = "finalize method not found on WorkingTree",
				instanceId = instance.id,
			}
		end
		return findings
	end,
	toSummary = function(_c, instance)
		return {
			kind = "working-tree",
			title = "Working tree",
			id = instance.ast.id,
			facts = {
				instance.ast.hasFinalize and "finalize" or "no finalize",
			},
		}
	end,
}
