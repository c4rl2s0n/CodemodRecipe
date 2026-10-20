--- Engine crate public surfaces: language adapters + query/engine entry types.

return {
	id = "repo.engine_surface",
	description = "Engine crate pub types/fns under src/ (language, query, span, ops)",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles("rust/crates/engine/src/**/*.rs")) do
			out[#out + 1] = { path = path }
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local identity = {
			componentId = "repo.engine_surface",
			id = path,
			path = path,
		}
		local children = {}
		local ok = pcall(function()
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
			local fnNames = c:query({
				path = path,
				language = "rust",
				query = "repo_rust.pub_functions",
				capture = "fnName",
			})
			local fns = c:query({
				path = path,
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
		if not ok then
			children = {}
		end
		return {
			id = path,
			componentId = "repo.engine_surface",
			source = { path = path },
			children = children,
		}
	end,
	relate = function(_c, instance, index)
		local edges = {
			{ kind = "lives_in", to = "repo.zone/platform.engine" },
		}
		local path = instance.ast and instance.ast.source and instance.ast.source.path or ""
		if index:byId("repo.engine_edit/engine-edit") then
			local kind = "uses"
			if path:find("/query/", 1, true) or path:find("/language/", 1, true) or path:match("ops%.rs$") then
				kind = "feeds"
			end
			edges[#edges + 1] = { kind = kind, to = "repo.engine_edit/engine-edit" }
		end
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "engine-surface",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " items" },
		}
	end,
}
