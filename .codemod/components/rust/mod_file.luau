--- Rust barrel / module file (lib.rs or mod.rs) with mod/use children.

return {
	id = "repo.rust_mod_file",
	description = "Rust module file (lib.rs / mod.rs): declares mod and use items",
	identify = function(c)
		local out = {}
		local seen = {}
		local function add(path)
			if not seen[path] then
				seen[path] = true
				out[#out + 1] = { path = path }
			end
		end
		for _, path in ipairs(c:listFiles("rust/crates/*/src/lib.rs")) do
			add(path)
		end
		for _, path in ipairs(c:listFiles("rust/crates/*/src/**/mod.rs")) do
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
			componentId = "repo.rust_mod_file",
			id = path,
			path = path,
		}
		local children = {}
		local modNames = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.mod_items",
			capture = "modName",
		})
		local modItems = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.mod_items",
			capture = "modItem",
		})
		table.sort(modNames, function(a, b)
			return a.span.from < b.span.from
		end)
		table.sort(modItems, function(a, b)
			return a.span.from < b.span.from
		end)
		for i = 1, math.min(#modNames, #modItems) do
			local child = c:component("repo.rust_mod_item"):parse({
				source = { path = path, span = modItems[i].span },
				parent = identity,
				role = "mod",
				hints = { kind = "mod", name = modNames[i].text },
			})
			if child then
				child.componentId = "repo.rust_mod_item"
				child.role = "mod"
				children[#children + 1] = child
			end
		end
		local uses = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.use_declarations",
			capture = "useItem",
		})
		for i, hit in ipairs(uses) do
			local child = c:component("repo.rust_mod_item"):parse({
				source = { path = path, span = hit.span },
				parent = identity,
				role = "use",
				hints = { kind = "use", name = "use#" .. i },
			})
			if child then
				child.componentId = "repo.rust_mod_item"
				child.role = "use"
				children[#children + 1] = child
			end
		end
		return {
			id = path,
			componentId = "repo.rust_mod_file",
			source = { path = path },
			children = children,
		}
	end,
	relate = function(_c, _instance, _index)
		-- Structural barrels: membership zone (ignoreRelations), not crate packaging.
		return { { kind = "lives_in", to = "repo.zone/structural.modules" } }
	end,
	toSummary = function(_c, instance)
		return {
			kind = "rust-mod-file",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " items" },
		}
	end,
}
