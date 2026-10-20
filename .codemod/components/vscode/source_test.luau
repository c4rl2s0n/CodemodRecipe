--- VS Code extension product tests related to production ts_modules.

local function productionPathFromTest(path)
	-- foo.test.ts → foo.ts / foo.vue
	local base = path:match("^(.*)%.test%.ts$") or path:match("^(.*)%.spec%.ts$")
	if not base then
		return nil
	end
	return { base .. ".ts", base .. ".vue" }
end

return {
	id = "repo.ts_test",
	description = "VS Code extension product test (*.test.ts / *.spec.ts)",
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
	parse = function(_c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		return {
			id = path,
			componentId = "repo.ts_test",
			source = { path = path },
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
			facts = {},
		}
	end,
}
