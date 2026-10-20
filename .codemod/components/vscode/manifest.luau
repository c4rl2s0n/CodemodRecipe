--- VS Code extension package.json: top-level fields + contributes sections.

local PACKAGE_JSON = "vscode_extension/package.json"

-- Contributes groups we model as named sections (coarse span of each pair).
local CONTRIBUTE_GROUPS = {
	commands = true,
	views = true,
	viewsContainers = true,
	configuration = true,
	keybindings = true,
	menus = true,
	submenus = true,
}

return {
	id = "repo.vscode_manifest",
	description = "VS Code extension package.json (top-level fields + contributes sections)",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles(PACKAGE_JSON)) do
			out[#out + 1] = { path = path }
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local identity = {
			componentId = "repo.vscode_manifest",
			id = "vscode-manifest",
			path = path,
		}
		local children = {}
		local ok = pcall(function()
			-- Top-level fields (skip contributes — handled as sections below).
			local keys = c:query({
				path = path,
				language = "json",
				query = "repo_json.top_level_pairs",
				capture = "key",
			})
			local pairs = c:query({
				path = path,
				language = "json",
				query = "repo_json.top_level_pairs",
				capture = "pair",
			})
			table.sort(keys, function(a, b)
				return a.span.from < b.span.from
			end)
			table.sort(pairs, function(a, b)
				return a.span.from < b.span.from
			end)
			for i = 1, math.min(#keys, #pairs) do
				local name = keys[i].text
				if name ~= "contributes" then
					local child = c:component("repo.json_field"):parse({
						source = { path = path, span = pairs[i].span },
						parent = identity,
						role = "top",
						hints = { name = name, role = "top" },
					})
					if child then
						child.componentId = "repo.json_field"
						child.role = "top"
						children[#children + 1] = child
					end
				end
			end

			-- contributes.<group> sections
			local groups = c:query({
				path = path,
				language = "json",
				query = "repo_json.contributes_groups",
				capture = "group",
			})
			local groupPairs = c:query({
				path = path,
				language = "json",
				query = "repo_json.contributes_groups",
				capture = "pair",
			})
			table.sort(groups, function(a, b)
				return a.span.from < b.span.from
			end)
			table.sort(groupPairs, function(a, b)
				return a.span.from < b.span.from
			end)
			for i = 1, math.min(#groups, #groupPairs) do
				local name = groups[i].text
				if CONTRIBUTE_GROUPS[name] then
					local child = c:component("repo.json_field"):parse({
						source = { path = path, span = groupPairs[i].span },
						parent = identity,
						role = "contributes",
						hints = { name = "contributes." .. name, role = "contributes" },
					})
					if child then
						child.componentId = "repo.json_field"
						child.role = "contributes"
						children[#children + 1] = child
					end
				end
			end
		end)
		if not ok then
			children = {}
		end

		local span = nil
		local rok, src = pcall(function()
			return c:read(path)
		end)
		if rok and type(src) == "string" then
			span = { from = 0, to = #src }
		end

		return {
			id = "vscode-manifest",
			componentId = "repo.vscode_manifest",
			source = { path = path, span = span },
			children = children,
		}
	end,
	relate = function(_c, instance, index)
		local edges = { { kind = "lives_in", to = "repo.zone/vscode.extension" } }
		for _, child in ipairs(instance.ast.children or {}) do
			if child.componentId == "repo.json_field" and child.id then
				local to = "repo.json_field/" .. child.id
				if index:byId(to) then
					edges[#edges + 1] = { kind = "defines", to = to }
				end
			end
		end
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "vscode-manifest",
			title = "package.json",
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " fields" },
		}
	end,
}
