--- Rust module file under crates/*/src (barrels + leaf modules) with mod/use children.

local function parentDir(path)
	return path:match("^(.*)/[^/]+$") or path
end

local function resolveModChild(dir, name)
	return {
		dir .. "/" .. name .. "/mod.rs",
		dir .. "/" .. name .. ".rs",
	}
end

return {
	id = "repo.rust_mod_file",
	description = "Rust source module under crates/*/src: declares_mod / uses for generation",
	identify = function(c)
		local out = {}
		local seen = {}
		local function add(path)
			-- Product tests are repo.rust_test; still include src/bin/*.rs (Cargo entrypoints).
			if path:match("_test%.rs$") or path:match("/tests/") then
				return
			end
			if not seen[path] then
				seen[path] = true
				out[#out + 1] = { path = path }
			end
		end
		for _, path in ipairs(c:listFiles("rust/crates/*/src/**/*.rs")) do
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
		local modNames = {}
		-- Best-effort child paint; keep partial results on failure (D-029).
		pcall(function()
			local names = c:query({
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
			table.sort(names, function(a, b)
				return a.span.from < b.span.from
			end)
			table.sort(modItems, function(a, b)
				return a.span.from < b.span.from
			end)
			for i = 1, math.min(#names, #modItems) do
				modNames[#modNames + 1] = names[i].text
				local child = c:component("repo.rust_mod_item"):parse({
					source = { path = path, span = modItems[i].span },
					parent = identity,
					role = "mod",
					hints = { kind = "mod", name = names[i].text },
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
			-- Cargo bins: paint fn leaves (not covered by host_module dirs).
			if path:find("/bin/", 1, true) then
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
					local child = c:component("repo.rust_mod_item"):parse({
						source = { path = path, span = fns[i].span },
						parent = identity,
						role = "fn",
						hints = { kind = "fn", name = fnNames[i].text },
					})
					if child then
						child.componentId = "repo.rust_mod_item"
						child.role = "fn"
						children[#children + 1] = child
					end
				end
			end
		end)
		-- crate::foo first segment for uses edges (text scan; stable enough for codegen anchors)
		local crateUses = {}
		local srcOk, src = pcall(function()
			return c:read(path)
		end)
		if srcOk and src then
			for seg in src:gmatch("use%s+crate::([%w_]+)") do
				crateUses[seg] = true
			end
		end
		return {
			id = path,
			componentId = "repo.rust_mod_file",
			source = { path = path },
			children = children,
			modNames = modNames,
			crateUses = crateUses,
		}
	end,
	relate = function(_c, instance, index)
		local edges = {
			{ kind = "lives_in", to = "repo.zone/structural.modules" },
		}
		local path = instance.ast and instance.ast.source and instance.ast.source.path
		if not path then
			return edges
		end
		local dir = parentDir(path)
		for _, name in ipairs(instance.ast.modNames or {}) do
			for _, cand in ipairs(resolveModChild(dir, name)) do
				local to = "repo.rust_mod_file/" .. cand
				if index:byId(to) then
					edges[#edges + 1] = { kind = "declares_mod", to = to }
					break
				end
			end
		end
		-- use crate::seg → sibling/child module under same crate src root
		local crateRoot = path:match("^(rust/crates/[^/]+/src)")
		if crateRoot then
			for seg, _ in pairs(instance.ast.crateUses or {}) do
				for _, cand in ipairs({
					crateRoot .. "/" .. seg .. "/mod.rs",
					crateRoot .. "/" .. seg .. ".rs",
					crateRoot .. "/" .. seg .. "/lib.rs",
				}) do
					local to = "repo.rust_mod_file/" .. cand
					if index:byId(to) and to ~= "repo.rust_mod_file/" .. path then
						edges[#edges + 1] = { kind = "uses", to = to }
						break
					end
				end
			end
		end
		return edges
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
