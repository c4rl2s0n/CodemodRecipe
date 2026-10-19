--- Logical unit: engine AST edit pipeline (Engine + EditOp + query apply).

local ENGINE_MOD = "rust/crates/engine/src/engine/mod.rs"
local OPS = "rust/crates/engine/src/ops.rs"

return {
	id = "repo.engine_edit",
	description = "Engine edit pipeline: Engine, EditOp, query-driven patches",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles(ENGINE_MOD)) do
			out[#out + 1] = { path = path }
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local engineSpan = nil
		local hasEngine = false
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
			if n.text == "Engine" then
				hasEngine = true
				engineSpan = items[i] and items[i].span
			end
		end
		local hasEditOp = false
		local editSpan = nil
		local opsOk = pcall(function()
			local onames = c:query({
				path = OPS,
				language = "rust",
				query = "repo_rust.type_items",
				capture = "typeName",
			})
			local oitems = c:query({
				path = OPS,
				language = "rust",
				query = "repo_rust.type_items",
				capture = "typeItem",
			})
			for i, n in ipairs(onames) do
				if n.text == "EditOp" then
					hasEditOp = true
					editSpan = oitems[i] and oitems[i].span
				end
			end
		end)
		local children = {}
		local identity = {
			componentId = "repo.engine_edit",
			id = "engine-edit",
			path = path,
		}
		if engineSpan then
			children[#children + 1] = {
				id = "Engine",
				componentId = "repo.host_module_item",
				source = { path = path, span = engineSpan },
				kind = "type",
				name = "Engine",
				role = "type",
			}
		end
		if editSpan then
			children[#children + 1] = {
				id = "EditOp",
				componentId = "repo.host_module_item",
				source = { path = OPS, span = editSpan },
				kind = "type",
				name = "EditOp",
				role = "type",
			}
		end
		return {
			id = "engine-edit",
			componentId = "repo.engine_edit",
			source = { path = path },
			hasEngine = hasEngine,
			hasEditOp = hasEditOp and opsOk,
			opsPath = OPS,
			children = children,
		}
	end,
	relate = function(_c, _instance, _index)
		return {
			{ kind = "lives_in", to = "repo.zone/platform.engine" },
			{
				kind = "produces",
				to = { symbol = "codemod_recipe_core::apply::file_change::FileChange" },
			},
		}
	end,
	verify = function(_c, instance)
		local findings = {}
		if not instance.ast.hasEngine then
			findings[#findings + 1] = {
				severity = "error",
				code = "engine_edit.missing_engine",
				message = "Engine struct not found in " .. ENGINE_MOD,
				instanceId = instance.id,
			}
		end
		if not instance.ast.hasEditOp then
			findings[#findings + 1] = {
				severity = "error",
				code = "engine_edit.missing_edit_op",
				message = "EditOp enum not found in " .. OPS,
				instanceId = instance.id,
			}
		end
		return findings
	end,
	toSummary = function(_c, instance)
		return {
			kind = "engine-edit",
			title = "Engine edits",
			id = instance.ast.id,
			facts = {
				instance.ast.hasEngine and "Engine" or "no Engine",
				instance.ast.hasEditOp and "EditOp" or "no EditOp",
			},
		}
	end,
}
