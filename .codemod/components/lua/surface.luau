--- Lua crate load/model surfaces (Asset kinds + pub types). Load grain: repo.lua_load_module.

return {
	id = "repo.lua_surface",
	description = "Lua crate src files: pub types/fns (load/ split tracked by repo.lua_load_module)",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles("rust/crates/lua/src/**/*.rs")) do
			-- load/ is owned by repo.lua_load_module (avoid double-cover).
			if not path:find("/load/", 1, true) then
				out[#out + 1] = { path = path }
			end
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local identity = {
			componentId = "repo.lua_surface",
			id = path,
			path = path,
		}
		local children = {}
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
		return {
			id = path,
			componentId = "repo.lua_surface",
			source = { path = path },
			children = children,
		}
	end,
	verify = function(_c, _instance)
		return {}
	end,
	relate = function(_c, _instance)
		return { { kind = "lives_in", to = "repo.zone/structural.modules" } }
	end,
	toSummary = function(_c, instance)
		return {
			kind = "lua-surface",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " items" },
		}
	end,
}
