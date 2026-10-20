--- Luau host API specs split by capability (post specs.rs grain).

local STAGE_HINTS = {
	["mod.rs"] = "barrel",
	["traits.rs"] = "traits",
	["query.rs"] = "query",
	["common.rs"] = "common",
	["edits.rs"] = "edits",
	["files.rs"] = "files",
	["queries.rs"] = "queries",
	["template.rs"] = "template",
}

-- Capability modules feed the host handler groups of the same name.
local FEEDS = {
	edits = "repo.host_api_handlers/edits",
	files = "repo.host_api_handlers/files",
	queries = "repo.host_api_handlers/queries",
	template = "repo.host_api_handlers/template_methods",
}

return {
	id = "repo.lua_host_api_specs",
	description = "Luau host API spec modules under rust/crates/lua/src/host_api/specs/",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles("rust/crates/lua/src/host_api/specs/**/*.rs")) do
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
		local stage = STAGE_HINTS[base] or base:gsub("%.rs$", "")
		local identity = {
			componentId = "repo.lua_host_api_specs",
			id = "specs." .. stage,
			path = path,
		}
		local children = {}
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
		pcall(function()
			local attrs = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.attribute_items",
				capture = "attr",
			})
			for i, hit in ipairs(attrs) do
				local child = c:component("repo.host_module_item"):parse({
					source = { path = path, span = hit.span },
					parent = identity,
					role = "attr",
					hints = { name = "attr#" .. i, kind = "attr" },
				})
				if child then
					child.componentId = "repo.host_module_item"
					child.role = "attr"
					children[#children + 1] = child
				end
			end
		end)
		return {
			id = "specs." .. stage,
			componentId = "repo.lua_host_api_specs",
			source = { path = path },
			stage = stage,
			children = children,
		}
	end,
	relate = function(_c, instance, index)
		local edges = {
			{ kind = "lives_in", to = "repo.zone/platform.lua" },
		}
		local stage = instance.ast.stage
		-- Barrel re-exports capability modules (honest public surface).
		if stage == "barrel" then
			for _, cap in ipairs({
				"traits",
				"query",
				"common",
				"edits",
				"files",
				"queries",
				"template",
			}) do
				local to = "repo.lua_host_api_specs/specs." .. cap
				if index:byId(to) then
					edges[#edges + 1] = { kind = "exports", to = to }
				end
			end
			return edges
		end
		-- Capability specs use shared traits + query/common building blocks.
		if stage ~= "traits" then
			if index:byId("repo.lua_host_api_specs/specs.traits") then
				edges[#edges + 1] = { kind = "uses", to = "repo.lua_host_api_specs/specs.traits" }
			end
		end
		if stage == "edits" or stage == "queries" then
			if index:byId("repo.lua_host_api_specs/specs.query") then
				edges[#edges + 1] = { kind = "uses", to = "repo.lua_host_api_specs/specs.query" }
			end
		end
		if stage == "edits" or stage == "files" or stage == "template" then
			if index:byId("repo.lua_host_api_specs/specs.common") then
				edges[#edges + 1] = { kind = "uses", to = "repo.lua_host_api_specs/specs.common" }
			end
		end
		-- Traits underpin every HostSpec/HostType adapter.
		if stage == "traits" then
			if index:byId("repo.lua_host_api_specs/specs.query") then
				edges[#edges + 1] = { kind = "feeds", to = "repo.lua_host_api_specs/specs.query" }
			end
			if index:byId("repo.lua_host_api_specs/specs.common") then
				edges[#edges + 1] = { kind = "feeds", to = "repo.lua_host_api_specs/specs.common" }
			end
		end
		local feedTo = FEEDS[stage]
		if feedTo and index:byId(feedTo) then
			edges[#edges + 1] = { kind = "feeds", to = feedTo }
		end
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "lua-host-api-specs",
			title = instance.ast.stage or instance.ast.id,
			id = instance.ast.id,
			facts = {
				#(instance.ast.children or {}) .. " leaves",
				instance.ast.source and instance.ast.source.path or "",
			},
		}
	end,
}
