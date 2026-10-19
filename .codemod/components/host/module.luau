--- Host src module directory (sandbox, preview, …) with pub fn / type children.

local zonesLib = require("../../lib/zones")

local MODULES = {
	{ id = "sandbox", path = "rust/crates/host/src/sandbox/mod.rs" },
	{ id = "preview", path = "rust/crates/host/src/preview/mod.rs" },
	{ id = "template", path = "rust/crates/host/src/template/mod.rs" },
	{ id = "query", path = "rust/crates/host/src/query/mod.rs" },
	{ id = "explorer", path = "rust/crates/host/src/explorer/mod.rs" },
	{ id = "diagnostics", path = "rust/crates/host/src/diagnostics/mod.rs" },
	{ id = "config", path = "rust/crates/host/src/config/mod.rs" },
	{ id = "host_api", path = "rust/crates/host/src/host_api/mod.rs" },
	{ id = "recipe", path = "rust/crates/host/src/recipe/mod.rs" },
	{ id = "protocol", path = "rust/crates/host/src/protocol/mod.rs" },
	{ id = "edit_session", path = "rust/crates/host/src/edit_session/mod.rs" },
}

return {
	id = "repo.host_module",
	description = "Host crate module directory: pub fn / type leaves for coverage",
	identify = function(c)
		local out = {}
		for _, mod in ipairs(MODULES) do
			for _, path in ipairs(c:listFiles(mod.path)) do
				out[#out + 1] = { path = path, hints = { moduleId = mod.id } }
			end
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		local moduleId = locator.hints and locator.hints.moduleId
		if not path or not moduleId then
			return nil
		end
		local dir = path:match("^(.*)/[^/]+$") or path
		local identity = {
			componentId = "repo.host_module",
			id = moduleId,
			path = path,
		}
		local children = {}
		-- Collect pub fns and type items from every .rs file under the module dir.
		local files = c:listFiles(dir .. "/**/*.rs")
		for _, fpath in ipairs(files) do
			local ok = pcall(function()
				local fnNames = c:query({
					path = fpath,
					language = "rust",
					query = "repo_rust.pub_functions",
					capture = "fnName",
				})
				local fns = c:query({
					path = fpath,
					language = "rust",
					query = "repo_rust.pub_functions",
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
						source = { path = fpath, span = fns[i].span },
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
				local typeNames = c:query({
					path = fpath,
					language = "rust",
					query = "repo_rust.type_items",
					capture = "typeName",
				})
				local types = c:query({
					path = fpath,
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
						source = { path = fpath, span = types[i].span },
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
			if not ok then
				-- skip unparseable
			end
		end
		return {
			id = moduleId,
			componentId = "repo.host_module",
			source = { path = path },
			children = children,
		}
	end,
	relate = function(_c, instance, index)
		local path = instance.ast and instance.ast.source and instance.ast.source.path
		local zone = zonesLib.hostZoneForPath(path)
		local edges = { { kind = "lives_in", to = "repo.zone/" .. zone } }
		local id = instance.ast and instance.ast.id
		-- Domain edges so host modules are architecture-connected, not membership-only.
		if id == "edit_session" then
			if index:byId("repo.engine_edit/engine-edit") then
				edges[#edges + 1] = { kind = "uses", to = "repo.engine_edit/engine-edit" }
			end
			if index:byId("repo.working_tree/working-tree") then
				edges[#edges + 1] = { kind = "stages_via", to = "repo.working_tree/working-tree" }
			end
		elseif id == "host_api" then
			if index:byId("repo.engine_edit/engine-edit") then
				edges[#edges + 1] = { kind = "drives", to = "repo.engine_edit/engine-edit" }
			end
			if index:byId("repo.working_tree/working-tree") then
				edges[#edges + 1] = { kind = "stages_via", to = "repo.working_tree/working-tree" }
			end
			if index:byId("repo.host_api_handlers/edits") then
				edges[#edges + 1] = { kind = "uses", to = "repo.host_api_handlers/edits" }
			end
		elseif id == "recipe" then
			if index:byId("repo.recipe_registry/recipe-registry") then
				edges[#edges + 1] = { kind = "feeds", to = "repo.recipe_registry/recipe-registry" }
			end
		elseif id == "query" then
			if index:byId("repo.engine_edit/engine-edit") then
				edges[#edges + 1] = { kind = "uses", to = "repo.engine_edit/engine-edit" }
			end
		end
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "host-module",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " items" },
		}
	end,
}
