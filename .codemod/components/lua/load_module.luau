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

local function deepenLeaves(c, path, identity)
	local children = {}
	-- Per-family pcall so one bad query cannot erase successful leaves.
	pcall(function()
		local fnNames = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.all_functions",
			capture = "fnName",
		})
		local fns = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.all_functions",
			capture = "fn",
		})
		table.sort(fnNames, function(a, b)
			return a.span.from < b.span.from
		end)
		table.sort(fns, function(a, b)
			return a.span.from < b.span.from
		end)
		for i = 1, math.min(#fnNames, #fns) do
			local child = c:component("repo.host_module_item"):parse({
				source = { path = path, span = fns[i].span },
				parent = identity,
				role = "fn",
				hints = { name = fnNames[i].text, kind = "fn" },
			})
			if child then
				child.componentId = "repo.host_module_item"
				child.role = "fn"
				children[#children + 1] = child
			end
		end
	end)
	pcall(function()
		local typeNames = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.type_items",
			capture = "typeName",
		})
		local types = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.type_items",
			capture = "typeItem",
		})
		table.sort(typeNames, function(a, b)
			return a.span.from < b.span.from
		end)
		table.sort(types, function(a, b)
			return a.span.from < b.span.from
		end)
		for i = 1, math.min(#typeNames, #types) do
			local child = c:component("repo.host_module_item"):parse({
				source = { path = path, span = types[i].span },
				parent = identity,
				role = "type",
				hints = { name = typeNames[i].text, kind = "type" },
			})
			if child then
				child.componentId = "repo.host_module_item"
				child.role = "type"
				children[#children + 1] = child
			end
		end
	end)
	local function paintSimple(queryId, capture, role)
		pcall(function()
			local hits = c:query({
				path = path,
				language = "rust",
				query = queryId,
				capture = capture,
			})
			for i, hit in ipairs(hits) do
				local child = c:component("repo.host_module_item"):parse({
					source = { path = path, span = hit.span },
					parent = identity,
					role = role,
					hints = { name = role .. "#" .. i, kind = role },
				})
				if child then
					child.componentId = "repo.host_module_item"
					child.role = role
					children[#children + 1] = child
				end
			end
		end)
	end
	paintSimple("repo_rust.use_items", "useItem", "use")
	paintSimple("repo_rust.macro_invocations", "macro", "macro")
	paintSimple("repo_rust.line_comments", "comment", "comment")
	return children
end

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
		local identity = {
			componentId = "repo.lua_load_module",
			id = "load." .. stage,
			path = path,
		}
		return {
			id = "load." .. stage,
			componentId = "repo.lua_load_module",
			source = { path = path },
			stage = stage,
			children = deepenLeaves(c, path, identity),
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
			facts = {
				#(instance.ast.children or {}) .. " leaves",
				instance.ast.source and instance.ast.source.path or "",
			},
		}
	end,
}
