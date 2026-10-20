--- #[cfg(test)] mod tests { … } under crates/*/src — bundles cases + helpers.

local childrenLib = require("../../lib/rust_test_children")
local zonesLib = require("../../lib/zones")

return {
	id = "repo.rust_test_module",
	description = "Rust unit-test module (mod tests) with test/helper leaves",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles("rust/crates/*/src/**/*.rs")) do
			local ok = pcall(function()
				local mods = c:query({
					path = path,
					language = "rust",
					query = "repo_rust.test_mod_items",
					capture = "modItem",
				})
				for _, hit in ipairs(mods) do
					out[#out + 1] = {
						path = path,
						hints = { span = hit.span },
					}
				end
			end)
			if not ok then
				-- skip unparseable
			end
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		local span = locator.hints and locator.hints.span
		if not path or not span then
			return nil
		end
		local identity = {
			componentId = "repo.rust_test_module",
			id = path .. ":mod:tests@" .. tostring(span.from),
			path = path,
		}
		local children = childrenLib.parseChildren(c, path, identity, span)
		return {
			id = identity.id,
			componentId = "repo.rust_test_module",
			source = { path = path, span = span },
			children = children,
			productionPath = path,
		}
	end,
	relate = function(_c, instance, index)
		local path = instance.ast and instance.ast.source and instance.ast.source.path or ""
		local zone = zonesLib.hostZoneForPath(path)
		if path:find("/engine/", 1, true) then
			zone = "platform.engine"
		elseif path:find("/lua/", 1, true) then
			zone = "platform.lua"
		elseif not path:find("/host/", 1, true) then
			zone = "domain.components"
		end
		local edges = {
			{ kind = "lives_in", to = "repo.zone/" .. zone },
		}
		local to = "repo.rust_mod_file/" .. path
		if index:byId(to) then
			edges[#edges + 1] = { kind = "tests", to = to }
		end
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "rust-test-module",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " leaves" },
		}
	end,
}
