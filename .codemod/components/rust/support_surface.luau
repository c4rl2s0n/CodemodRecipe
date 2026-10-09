--- Core + query_tools + lua_macros: pub surface leaves (thin crates).

local GLOBS = {
	"rust/crates/core/src/**/*.rs",
	"rust/crates/query_tools/src/**/*.rs",
	"rust/crates/lua_macros/src/**/*.rs",
}

return {
	id = "repo.support_crate_surface",
	description = "core / query_tools / lua_macros pub types and functions",
	identify = function(c)
		local out, seen = {}, {}
		for _, pattern in ipairs(GLOBS) do
			for _, path in ipairs(c:listFiles(pattern)) do
				if not seen[path] then
					seen[path] = true
					out[#out + 1] = { path = path }
				end
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
			componentId = "repo.support_crate_surface",
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
			componentId = "repo.support_crate_surface",
			source = { path = path },
			children = children,
		}
	end,
	relate = function(_c, _instance)
		return { { kind = "lives_in", to = "repo.zone/structural.modules" } }
	end,
	toSummary = function(_c, instance)
		return {
			kind = "support-crate-surface",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " items" },
		}
	end,
}
