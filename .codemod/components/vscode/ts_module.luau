--- TypeScript / Vue module file with top-level export children + import edges.

local function parentDir(path)
	return path:match("^(.*)/[^/]+$") or ""
end

local function resolveImport(fromPath, spec)
	if not spec or spec:sub(1, 1) ~= "." then
		return nil
	end
	local dir = parentDir(fromPath)
	-- normalize ./ and ../
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
	if #parts == 0 then
		return nil
	end
	local last = parts[#parts]
	last = last:gsub("%.vue$", ""):gsub("%.tsx?$", ""):gsub("%.jsx?$", "")
	parts[#parts] = last
	local base = table.concat(parts, "/")
	return {
		base .. ".ts",
		base .. ".vue",
		base .. "/index.ts",
		base .. "/index.vue",
	}
end


return {
	id = "repo.ts_module",
	description = "TS source module: exports as coverage leaves; imports for generation graph (.vue → repo.vue_sfc)",
	identify = function(c)
		local out = {}
		local seen = {}
		local function addPath(path)
			-- Product tests are repo.ts_test; bootstrap owns webview main.ts; SFCs are repo.vue_sfc.
			if path:match("%.test%.ts$") or path:match("%.spec%.ts$") or path:match("%.vue$") then
				return
			end
			-- Mirror stack.codebase ! excludes (listFiles does not inherit codebase).
			if path:match("%.d%.ts$") or path:match("%.config%.ts$") then
				return
			end
			if path == "vscode_extension/src/webview/src/main.ts" then
				return
			end
			-- Domain composers own these paths (avoid double-identify).
			if path:match("/extension/language/context/") then
				return
			end
			if path:match("/extension/bootstrap/") then
				return
			end
			if path:match("/recipes/recipeInvoke%.ts$") or path:match("/recipes/recipeKeybindings%.ts$") then
				return
			end
			if path == "vscode_extension/src/extension/host/hostBridge.ts" then
				return
			end
			if path == "vscode_extension/src/shared/messages.ts" then
				return
			end
			if path == "vscode_extension/src/extension/constants.ts" then
				return
			end
			if not seen[path] then
				seen[path] = true
				out[#out + 1] = { path = path }
			end
		end

		for _, path in ipairs(c:listFiles("vscode_extension/src/**/*.ts")) do
			addPath(path)
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local language = "typescript"
		local identity = {
			componentId = "repo.ts_module",
			id = path,
			path = path,
		}
		local children = {}
		local importSpecs = {}
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
		local parseError = nil
		local ok, err = pcall(function()
			addExports("repo_typescript.export_lexical", "exportName")
			addExports("repo_typescript.export_function", "exportName")
			addExports("repo_typescript.export_class", "exportName")
			addExports("repo_typescript.export_type", "exportName")
			local imports = c:query({
				path = path,
				language = language,
				query = "repo_typescript.import_sources",
				capture = "importPath",
			})
			for _, hit in ipairs(imports) do
				if hit.text and hit.text ~= "" then
					importSpecs[#importSpecs + 1] = hit.text
				end
			end
		end)
		if not ok then
			parseError = tostring(err)
		end
		if #children == 0 and not parseError then
			local stmtsOk = pcall(function()
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
			end)
			if not stmtsOk then
				-- leave children empty; Vue fallback below may recover
			end
		end
		-- Deepen: local/helper fns + class methods (span; overlaps with exports merge in coverage).
		local function addNamedFns(queryId, nameCap, bodyCap, role)
			local names = c:query({
				path = path,
				language = language,
				query = queryId,
				capture = nameCap,
			})
			local bodies = c:query({
				path = path,
				language = language,
				query = queryId,
				capture = bodyCap,
			})
			table.sort(names, function(a, b)
				return a.span.from < b.span.from
			end)
			table.sort(bodies, function(a, b)
				return a.span.from < b.span.from
			end)
			for i = 1, math.min(#names, #bodies) do
				local child = c:component("repo.ts_export"):parse({
					source = { path = path, span = bodies[i].span },
					parent = identity,
					role = role,
					hints = { name = names[i].text },
				})
				if child then
					child.componentId = "repo.ts_export"
					child.role = role
					children[#children + 1] = child
				end
			end
		end
		pcall(function()
			addNamedFns("repo_typescript.function_declarations", "fnName", "fn", "fn")
		end)
		pcall(function()
			addNamedFns("repo_typescript.const_function_bindings", "fnName", "fn", "fn")
		end)
		pcall(function()
			addNamedFns("repo_typescript.bridge_methods", "methodName", "method", "method")
		end)
		-- Import statements as span leaves (prefix gap on activate / providers).
		pcall(function()
			local imports = c:query({
				path = path,
				language = language,
				query = "repo_typescript.import_statements",
				capture = "import",
			})
			for i, hit in ipairs(imports) do
				local child = c:component("repo.ts_export"):parse({
					source = { path = path, span = hit.span },
					parent = identity,
					role = "import",
					hints = { name = "import#" .. i },
				})
				if child then
					child.componentId = "repo.ts_export"
					child.role = "import"
					children[#children + 1] = child
				end
			end
		end)
		local reexportSpecs = {}
		local srcOk, src = pcall(function()
			return c:read(path)
		end)
		if srcOk and src then
			-- Text fallback for imports if query missed (Vue / string shape)
			if #importSpecs == 0 then
				for spec in src:gmatch("from%s*[\"'](%.[^\"']+)[\"']") do
					importSpecs[#importSpecs + 1] = spec
				end
			end
			-- Barrel re-exports: export … from './foo'
			if path:match("/index%.ts$") then
				for spec in src:gmatch("export%s+.-from%s*[\"'](%.[^\"']+)[\"']") do
					reexportSpecs[#reexportSpecs + 1] = spec
				end
			end
		end
		return {
			id = path,
			componentId = "repo.ts_module",
			source = { path = path },
			children = children,
			importSpecs = importSpecs,
			reexportSpecs = reexportSpecs,
			parseError = parseError,
		}
	end,
	operations = {
		scaffoldTest = {
			description = "Create a vitest stub *.test.ts beside this module (product test graph edge)",
			args = {
				{ name = "ifExists", required = false, help = "skip (default) | error" },
			},
			apply = function(c, args, target, _graph)
				local path = target and target.ast and target.ast.source and target.ast.source.path
					or target and target.ast and target.ast.id
				if not path or not path:match("%.ts$") or path:match("%.vue$") then
					error("scaffoldTest requires a .ts module target")
				end
				local testPath = path:gsub("%.ts$", ".test.ts")
				local stem = path:match("([^/]+)%.ts$") or "module"
				local content = table.concat({
					"import { describe, expect, it } from 'vitest';",
					"",
					"describe('" .. stem .. "', () => {",
					"  it('placeholder — replace with real assertions', () => {",
					"    expect(true).toBe(true);",
					"  });",
					"});",
					"",
				}, "\n")
				c:create({
					path = testPath,
					content = content,
					ifExists = args.ifExists or "skip",
				})
			end,
		},
	},
	relate = function(_c, instance, index)
		local zones = require("../../lib/zones")
		local path = instance.ast and instance.ast.source and instance.ast.source.path
			or instance.ast and instance.ast.id
		local zone = zones.vscodeZoneForPath(path)
		local edges = { { kind = "lives_in", to = "repo.zone/" .. zone } }
		local DOMAIN_PREFIXES = {
			"repo.ts_module/",
			"repo.vscode_language_context/",
			"repo.vscode_ext_bootstrap/",
			"repo.recipe_invoke/",
			"repo.vue_sfc/",
		}
		local function linkSpecs(specs, kind)
			local seenTo = {}
			for _, spec in ipairs(specs or {}) do
				local cands = resolveImport(path, spec)
				if cands then
					for _, cand in ipairs(cands) do
						local linked = false
						for _, prefix in ipairs(DOMAIN_PREFIXES) do
							local to = prefix .. cand
							if index:byId(to) and not seenTo[to] then
								seenTo[to] = true
								edges[#edges + 1] = { kind = kind, to = to }
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
		end

		linkSpecs(instance.ast.importSpecs, "imports")
		-- Barrel index.ts: exports → re-exported siblings (scaffold append target).
		if path:match("/index%.ts$") then
			linkSpecs(instance.ast.reexportSpecs, "exports")
		end
		return edges
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
