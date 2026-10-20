--- Per-asset load modules under lua/src/load/ (post D-025 A1 grain).

local STAGE_HINTS = {
	["mod.rs"] = "dispatch",
	["recipe.rs"] = "recipe",
	["map.rs"] = "map",
	["variables.rs"] = "variables",
	["query_lib.rs"] = "query_lib",
	["type_def.rs"] = "type_def",
	["component.rs"] = "component",
	["stack.rs"] = "stack",
	["test_suite.rs"] = "test_suite",
	["table_util.rs"] = "table_util",
	["shared.rs"] = "shared",
	["ops.rs"] = "ops",
	["type_parse.rs"] = "type_parse",
	["explorer.rs"] = "explorer",
	["error.rs"] = "error",
}

return {
	id = "repo.lua_load_module",
	description = "Luau asset load stages under rust/crates/lua/src/load/",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles("rust/crates/lua/src/load/**/*.rs")) do
			out[#out + 1] = { path = path }
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local base = path:match("([^/]+)$") or path
		local stage = STAGE_HINTS[base] or base
		return {
			id = "load." .. stage,
			componentId = "repo.lua_load_module",
			source = { path = path },
			stage = stage,
		}
	end,
	relate = function(_c, instance, index)
		local edges = {
			{ kind = "lives_in", to = "repo.zone/platform.lua" },
		}
		-- Dispatch facade feeds host recipe registry (load → apply pipeline).
		if instance.ast.stage == "dispatch" and index:byId("repo.recipe_registry/recipe-registry") then
			edges[#edges + 1] = { kind = "feeds", to = "repo.recipe_registry/recipe-registry" }
		end
		-- Asset-kind stages use the shared table/error helpers.
		local helpers = {
			table_util = true,
			error = true,
			shared = true,
			ops = true,
			type_parse = true,
			explorer = true,
		}
		if not helpers[instance.ast.stage] then
			if index:byId("repo.lua_load_module/load.table_util") then
				edges[#edges + 1] = { kind = "uses", to = "repo.lua_load_module/load.table_util" }
			end
			if index:byId("repo.lua_load_module/load.error") then
				edges[#edges + 1] = { kind = "uses", to = "repo.lua_load_module/load.error" }
			end
			if index:byId("repo.lua_load_module/load.shared") then
				edges[#edges + 1] = { kind = "uses", to = "repo.lua_load_module/load.shared" }
			end
		elseif instance.ast.stage == "ops" or instance.ast.stage == "type_parse" or instance.ast.stage == "explorer" then
			if index:byId("repo.lua_load_module/load.table_util") then
				edges[#edges + 1] = { kind = "uses", to = "repo.lua_load_module/load.table_util" }
			end
			if index:byId("repo.lua_load_module/load.shared") then
				edges[#edges + 1] = { kind = "uses", to = "repo.lua_load_module/load.shared" }
			end
		end
		return edges
	end,
	verify = function(_c, _instance)
		return {}
	end,
	toSummary = function(_c, instance)
		return {
			kind = "lua-load-module",
			title = instance.ast.stage or instance.ast.id,
			id = instance.ast.id,
			facts = { instance.ast.source and instance.ast.source.path or "" },
		}
	end,
}
