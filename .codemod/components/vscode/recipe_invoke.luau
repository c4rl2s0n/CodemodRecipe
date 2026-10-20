--- Recipe invoke + keybinding JSON helpers (extension command fan-in).

local PATHS = {
	"vscode_extension/src/extension/recipes/recipeInvoke.ts",
	"vscode_extension/src/extension/recipes/recipeKeybindings.ts",
}

return {
	id = "repo.recipe_invoke",
	description = "Recipe invoke / slot invoke and keybinding JSON formatters",
	identify = function(c)
		local out = {}
		for _, pattern in ipairs(PATHS) do
			for _, path in ipairs(c:listFiles(pattern)) do
				out[#out + 1] = { path = path }
			end
		end
		return out
	end,
	parse = function(c, locator)
		local ast = c:component("repo.ts_module"):parse(locator)
		if not ast then
			return nil
		end
		ast.componentId = "repo.recipe_invoke"
		return ast
	end,
	relate = function(_c, instance, index)
		local path = instance.ast and instance.ast.source and instance.ast.source.path
			or instance.ast and instance.ast.id
		local edges = { { kind = "lives_in", to = "repo.zone/vscode.extension" } }
		if path:match("recipeInvoke%.ts$") and index:byId("repo.host_bridge/host-bridge") then
			edges[#edges + 1] = { kind = "uses", to = "repo.host_bridge/host-bridge" }
		end
		-- Keybindings module depends on invoke mode vocabulary.
		if path:match("recipeKeybindings%.ts$") then
			local to = "repo.recipe_invoke/vscode_extension/src/extension/recipes/recipeInvoke.ts"
			if index:byId(to) then
				edges[#edges + 1] = { kind = "uses", to = to }
			end
		end

		local seen = {}
		for _, spec in ipairs(instance.ast.importSpecs or {}) do
			if spec:sub(1, 1) == "." then
				local dir = path:match("^(.*)/[^/]+$") or ""
				local parts = {}
				for seg in (dir .. "/" .. spec):gmatch("[^/]+") do
					if seg == ".." then
						if #parts > 0 then
							table.remove(parts)
						end
					elseif seg ~= "." and seg ~= "" then
						parts[#parts + 1] = seg
					end
				end
				local base = table.concat(parts, "/")
				for _, cand in ipairs({ base .. ".ts", base .. "/index.ts" }) do
					local toInv = "repo.recipe_invoke/" .. cand
					local toTs = "repo.ts_module/" .. cand
					local to = nil
					if index:byId(toInv) then
						to = toInv
					elseif index:byId(toTs) then
						to = toTs
					end
					if to and not seen[to] then
						seen[to] = true
						edges[#edges + 1] = { kind = "uses", to = to }
						break
					end
				end
			end
		end
		return edges
	end,
	toSummary = function(_c, instance)
		local path = instance.ast.id or ""
		local stem = path:match("([^/]+)%.ts$") or path
		return {
			kind = "recipe-invoke",
			title = stem,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " exports" },
		}
	end,
}
