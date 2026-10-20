--- Integration tests under crates/*/tests (file root + case/helper leaves).

local childrenLib = require("../../lib/rust_test_children")

local function productionCandidates(path)
	local crate = path:match("^(rust/crates/[^/]+)/")
	if not crate then
		return {}
	end
	local out = { crate .. "/src/lib.rs", crate .. "/src/main.rs" }
	local stem = path:match("/([^/]+)_test%.rs$") or path:match("/([^/]+)%.rs$")
	if stem and stem ~= "mod" and stem ~= "common" then
		local dir = path:match("^(.*)/[^/]+$") or ""
		local srcDir = dir:gsub("/tests$", "/src")
		out[#out + 1] = srcDir .. "/" .. stem .. ".rs"
		out[#out + 1] = srcDir .. "/" .. stem .. "/mod.rs"
	end
	return out
end

return {
	id = "repo.rust_test",
	description = "Rust integration test file (crates/*/tests/**/*.rs) with test/helper leaves",
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
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local identity = {
			componentId = "repo.rust_test",
			id = path,
			path = path,
		}
		local children = childrenLib.parseChildren(c, path, identity, nil)
		local span = nil
		local ok, src = pcall(function()
			return c:read(path)
		end)
		if ok and type(src) == "string" then
			span = { from = 0, to = #src }
		end
		return {
			id = path,
			componentId = "repo.rust_test",
			source = { path = path, span = span },
			children = children,
			productionCandidates = productionCandidates(path),
		}
	end,
	relate = function(_c, instance, index)
		local path = instance.ast and instance.ast.source and instance.ast.source.path or ""
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
			facts = { #(instance.ast.children or {}) .. " leaves" },
		}
	end,
}
