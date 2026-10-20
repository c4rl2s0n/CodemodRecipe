--- VS Code extension product tests with it/test/describe + helper leaves.

local function productionPathFromTest(path)
	local base = path:match("^(.*)%.test%.ts$") or path:match("^(.*)%.spec%.ts$")
	if not base then
		return nil
	end
	return { base .. ".ts", base .. ".vue" }
end

local function parseChildren(c, path, parent)
	local children = {}
	local ok = pcall(function()
		local function addCalls(queryId, kind)
			local calls = c:query({
				path = path,
				language = "typescript",
				query = queryId,
				capture = "call",
			})
			for i, hit in ipairs(calls) do
				local child = c:component("repo.ts_test_case"):parse({
					source = { path = path, span = hit.span },
					parent = parent,
					role = kind,
					hints = { name = kind .. "#" .. i, kind = kind },
				})
				if child then
					child.componentId = "repo.ts_test_case"
					child.role = kind
					children[#children + 1] = child
				end
			end
		end
		addCalls("repo_typescript.test_it_calls", "test")
		addCalls("repo_typescript.test_describe_calls", "describe")
		local fnNames = c:query({
			path = path,
			language = "typescript",
			query = "repo_typescript.function_declarations",
			capture = "fnName",
		})
		local fns = c:query({
			path = path,
			language = "typescript",
			query = "repo_typescript.function_declarations",
			capture = "fn",
		})
		table.sort(fnNames, function(a, b)
			return a.span.from < b.span.from
		end)
		table.sort(fns, function(a, b)
			return a.span.from < b.span.from
		end)
		for i = 1, math.min(#fnNames, #fns) do
			local child = c:component("repo.ts_test_case"):parse({
				source = { path = path, span = fns[i].span },
				parent = parent,
				role = "helper",
				hints = { name = fnNames[i].text, kind = "helper" },
			})
			if child then
				child.componentId = "repo.ts_test_case"
				child.role = "helper"
				children[#children + 1] = child
			end
		end
	end)
	if not ok then
		return {}
	end
	return children
end

return {
	id = "repo.ts_test",
	description = "VS Code extension product test (*.test.ts / *.spec.ts) with it/describe leaves",
	identify = function(c)
		local out = {}
		local seen = {}
		local function add(path)
			if not seen[path] then
				seen[path] = true
				out[#out + 1] = { path = path }
			end
		end
		for _, path in ipairs(c:listFiles("vscode_extension/src/**/*.test.ts")) do
			add(path)
		end
		for _, path in ipairs(c:listFiles("vscode_extension/src/**/*.spec.ts")) do
			add(path)
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local identity = {
			componentId = "repo.ts_test",
			id = path,
			path = path,
		}
		local span = nil
		local ok, src = pcall(function()
			return c:read(path)
		end)
		if ok and type(src) == "string" then
			span = { from = 0, to = #src }
		end
		return {
			id = path,
			componentId = "repo.ts_test",
			source = { path = path, span = span },
			children = parseChildren(c, path, identity),
			productionCandidates = productionPathFromTest(path),
		}
	end,
	relate = function(_c, instance, index)
		local zones = require("../../lib/zones")
		local path = instance.ast and instance.ast.source and instance.ast.source.path
		local zone = zones.vscodeZoneForPath(path)
		local edges = { { kind = "lives_in", to = "repo.zone/" .. zone } }
		for _, cand in ipairs(instance.ast.productionCandidates or {}) do
			local to = "repo.ts_module/" .. cand
			if index:byId(to) then
				edges[#edges + 1] = { kind = "tests", to = to }
				break
			end
		end
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "ts-test",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " leaves" },
		}
	end,
}
