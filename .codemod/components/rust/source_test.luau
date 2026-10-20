--- Product Rust integration/unit tests related to production modules.

local function productionCandidates(path)
	local crate = path:match("^(rust/crates/[^/]+)/")
	if not crate then
		return {}
	end
	-- Integration tests → crate lib; src/*_test.rs → sibling module stem.
	local out = { crate .. "/src/lib.rs", crate .. "/src/main.rs" }
	local stem = path:match("/([^/]+)_test%.rs$")
	if stem then
		local dir = path:match("^(.*)/[^/]+$") or ""
		-- Prefer co-located production module under src/
		local srcDir = dir:gsub("/tests$", "/src")
		out[#out + 1] = srcDir .. "/" .. stem .. ".rs"
		out[#out + 1] = srcDir .. "/" .. stem .. "/mod.rs"
	end
	return out
end

return {
	id = "repo.rust_test",
	description = "Rust product test file (*_test.rs or crates/*/tests/*.rs)",
	identify = function(c)
		local out = {}
		local seen = {}
		local function add(path)
			if not seen[path] then
				seen[path] = true
				out[#out + 1] = { path = path }
			end
		end
		for _, path in ipairs(c:listFiles("rust/crates/*/tests/**/*.rs")) do
			add(path)
		end
		for _, path in ipairs(c:listFiles("rust/crates/*/src/**/*_test.rs")) do
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
			componentId = "repo.rust_test",
			source = { path = path },
			productionCandidates = productionCandidates(path),
		}
	end,
	relate = function(_c, instance, index)
		local path = instance.ast and instance.ast.source and instance.ast.source.path or ""
		-- Architecture zone (not structural.modules — that ignores relations).
		local zone = "domain.components"
		if path:find("/engine/", 1, true) then
			zone = "platform.engine"
		elseif path:find("/lua/", 1, true) then
			zone = "platform.lua"
		elseif path:find("/host/", 1, true) then
			local zonesLib = require("../../lib/zones")
			zone = zonesLib.hostZoneForPath(path:gsub("/tests/", "/src/"))
		end
		local edges = {
			{ kind = "lives_in", to = "repo.zone/" .. zone },
		}
		for _, cand in ipairs(instance.ast.productionCandidates or {}) do
			local to = "repo.rust_mod_file/" .. cand
			if index:byId(to) then
				edges[#edges + 1] = { kind = "tests", to = to }
				break
			end
		end
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "rust-test",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = {},
		}
	end,
}
