--- Extension activate helpers: workspace setup + recipe/slot pickers.

local GLOB = "vscode_extension/src/extension/bootstrap/**/*.ts"

return {
	id = "repo.vscode_ext_bootstrap",
	description = "Extension activate support (workspace setup, pickers) — not webview main.ts",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles(GLOB)) do
			if not path:match("/index%.ts$") then
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
		ast.componentId = "repo.vscode_ext_bootstrap"
		return ast
	end,
	relate = function(_c, instance, index)
		local path = instance.ast and instance.ast.source and instance.ast.source.path
			or instance.ast and instance.ast.id
		local edges = { { kind = "lives_in", to = "repo.zone/vscode.extension" } }
		local seen = {}
		local DOMAIN_PREFIXES = {
			"repo.vscode_ext_bootstrap/",
			"repo.recipe_invoke/",
			"repo.host_bridge/",
			"repo.ts_module/",
		}
		-- Lifecycle wires the host bridge (ensureHost / bootstrapProject).
		if path:match("lifecycle%.ts$") and index:byId("repo.host_bridge/host-bridge") then
			edges[#edges + 1] = { kind = "uses", to = "repo.host_bridge/host-bridge" }
		end
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
					local linked = false
					for _, prefix in ipairs(DOMAIN_PREFIXES) do
						local to = prefix .. cand
						if index:byId(to) and not seen[to] then
							seen[to] = true
							edges[#edges + 1] = { kind = "uses", to = to }
							linked = true
							break
						end
					end
					if linked then
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
			kind = "vscode-ext-bootstrap",
			title = stem,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " exports" },
		}
	end,
}
