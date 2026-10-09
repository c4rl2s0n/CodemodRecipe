--- TypeScript / Vue module file with top-level export children.

return {
	id = "repo.ts_module",
	description = "TS/Vue source module: top-level exports as coverage leaves",
	identify = function(c)
		local out = {}
		local seen = {}
		local function addAll(pattern)
			for _, path in ipairs(c:listFiles(pattern)) do
				if not seen[path] then
					seen[path] = true
					out[#out + 1] = { path = path }
				end
			end
		end
		addAll("vscode_extension/src/**/*.ts")
		addAll("vscode_extension/src/**/*.vue")
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		-- Vue SFCs: query language still typescript for script; skip if no exports.
		local language = path:match("%.vue$") and "typescript" or "typescript"
		local identity = {
			componentId = "repo.ts_module",
			id = path,
			path = path,
		}
		local children = {}
		local function addExports(queryId, captureName)
			local names = c:query({
				path = path,
				language = language,
				query = queryId,
				capture = captureName,
			})
			local exports = c:query({
				path = path,
				language = language,
				query = queryId,
				capture = "export",
			})
			table.sort(names, function(a, b)
				return a.span.from < b.span.from
			end)
			table.sort(exports, function(a, b)
				return a.span.from < b.span.from
			end)
			for i = 1, math.min(#names, #exports) do
				local child = c:component("repo.ts_export"):parse({
					source = { path = path, span = exports[i].span },
					parent = identity,
					role = "export",
					hints = { name = names[i].text },
				})
				if child then
					child.componentId = "repo.ts_export"
					child.role = "export"
					children[#children + 1] = child
				end
			end
		end
		local ok, err = pcall(function()
			addExports("repo_typescript.export_lexical", "exportName")
			addExports("repo_typescript.export_function", "exportName")
			addExports("repo_typescript.export_class", "exportName")
			addExports("repo_typescript.export_type", "exportName")
		end)
		if not ok then
			-- Vue / parse failures: still emit path-only root (no coverage leaf).
			return {
				id = path,
				componentId = "repo.ts_module",
				source = { path = path },
				children = {},
				parseError = tostring(err),
			}
		end
		-- Fallback: bare export_statement spans when named captures missed
		if #children == 0 then
			local stmts = c:query({
				path = path,
				language = language,
				query = "repo_typescript.export_statements",
				capture = "export",
			})
			for i, hit in ipairs(stmts) do
				local child = c:component("repo.ts_export"):parse({
					source = { path = path, span = hit.span },
					parent = identity,
					role = "export",
					hints = { name = "export#" .. i },
				})
				if child then
					child.componentId = "repo.ts_export"
					child.role = "export"
					children[#children + 1] = child
				end
			end
		end
		return {
			id = path,
			componentId = "repo.ts_module",
			source = { path = path },
			children = children,
		}
	end,
	relate = function(_c, instance)
		local zones = require("../../lib/zones")
		local path = instance.ast and instance.ast.source and instance.ast.source.path
			or instance.ast and instance.ast.id
		local zone = zones.vscodeZoneForPath(path)
		return { { kind = "lives_in", to = "repo.zone/" .. zone } }
	end,
	toSummary = function(_c, instance)
		return {
			kind = "ts-module",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " exports" },
		}
	end,
}
