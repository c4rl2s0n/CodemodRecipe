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
	-- Leaf file (not a directory mod): high-gap helpers for component tests.
	{ id = "testing", path = "rust/crates/host/src/testing.rs" },
	-- Leaf file: Luau ctx build / nested run (orphan from dir MODULES).
	{ id = "lua_runtime", path = "rust/crates/host/src/lua_runtime.rs" },
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
		local identity = {
			componentId = "repo.host_module",
			id = moduleId,
			path = path,
		}
		local children = {}
		-- Directory mods: all .rs under the dir. Leaf file modules: only that path.
		local files
		if path:match("/mod%.rs$") then
			local dir = path:match("^(.*)/[^/]+$") or path
			files = c:listFiles(dir .. "/**/*.rs")
		else
			files = { path }
		end
		for _, fpath in ipairs(files) do
			local ok = pcall(function()
				local fnNames = c:query({
					path = fpath,
					language = "rust",
					query = "repo_rust.all_functions",
					capture = "fnName",
				})
				local fns = c:query({
					path = fpath,
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
				local constNames = c:query({
					path = fpath,
					language = "rust",
					query = "repo_rust.pub_str_consts",
					capture = "constName",
				})
				local consts = c:query({
					path = fpath,
					language = "rust",
					query = "repo_rust.pub_str_consts",
					capture = "constItem",
				})
				table.sort(constNames, function(a, b)
					return a.span.from < b.span.from
				end)
				table.sort(consts, function(a, b)
					return a.span.from < b.span.from
				end)
				for i = 1, math.min(#constNames, #consts) do
					local child = c:component("repo.host_module_item"):parse({
						source = { path = fpath, span = consts[i].span },
						parent = identity,
						role = "const",
						hints = { name = constNames[i].text, kind = "const" },
					})
					if child then
						child.componentId = "repo.host_module_item"
						child.role = "const"
						children[#children + 1] = child
					end
				end
				-- Outer attrs (#[host_method], #[derive], …) sit between items.
				local attrs = c:query({
					path = fpath,
					language = "rust",
					query = "repo_rust.attribute_items",
					capture = "attr",
				})
				for i, hit in ipairs(attrs) do
					local child = c:component("repo.host_module_item"):parse({
						source = { path = fpath, span = hit.span },
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
		elseif id == "preview" then
			if index:byId("repo.host_module/edit_session") then
				edges[#edges + 1] = { kind = "uses", to = "repo.host_module/edit_session" }
			end
			if index:byId("repo.working_tree/working-tree") then
				edges[#edges + 1] = { kind = "stages_via", to = "repo.working_tree/working-tree" }
			end
		elseif id == "protocol" then
			if index:byId("repo.host_protocol/host-protocol") then
				edges[#edges + 1] = { kind = "uses", to = "repo.host_protocol/host-protocol" }
			end
		elseif id == "sandbox" then
			if index:byId("repo.host_module/host_api") then
				edges[#edges + 1] = { kind = "uses", to = "repo.host_module/host_api" }
			end
		elseif id == "config" then
			if index:byId("repo.host_module/recipe") then
				edges[#edges + 1] = { kind = "feeds", to = "repo.host_module/recipe" }
			end
		elseif id == "template" then
			if index:byId("repo.host_api_handlers/template_methods") then
				edges[#edges + 1] = { kind = "uses", to = "repo.host_api_handlers/template_methods" }
			end
		elseif id == "explorer" then
			if index:byId("repo.host_module/recipe") then
				edges[#edges + 1] = { kind = "uses", to = "repo.host_module/recipe" }
			end
		elseif id == "diagnostics" then
			if index:byId("repo.host_module/protocol") then
				edges[#edges + 1] = { kind = "uses", to = "repo.host_module/protocol" }
			end
		elseif id == "testing" then
			if index:byId("repo.component_pipeline/coverage") then
				edges[#edges + 1] = { kind = "uses", to = "repo.component_pipeline/coverage" }
			end
		elseif id == "lua_runtime" then
			if index:byId("repo.host_module/host_api") then
				edges[#edges + 1] = { kind = "uses", to = "repo.host_module/host_api" }
			end
			if index:byId("repo.host_api_method/host-api-surface") then
				edges[#edges + 1] = { kind = "uses", to = "repo.host_api_method/host-api-surface" }
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
