local function unquote(text)
	if not text then
		return nil
	end
	return (text:gsub('^"', ""):gsub('"$', ""))
end

--- Expected path-dep edges (crate dir → set of allowed dependency dirs).
--- See docs/develop/05_dogfood/05_target_architecture.md.
	local ALLOWED_PATH_DEPS = {
	lua_macros = {},
	core = {},
	-- D-016: shared QuerySpec/IfExists shapes live in core; lua FromLua adapters depend on core.
	lua = { lua_macros = true, core = true },
	engine = { core = true },
	query_tools = { engine = true },
	-- D-025 F: component domain + ComponentHost facade (no SharedHost).
	components = { core = true, lua = true },
	host = {
		core = true,
		components = true,
		engine = true,
		lua = true,
		lua_macros = true,
		query_tools = true,
	},
}

local function pathDeps(c, cargoToml)
	local deps = {}
	local src = c:read(cargoToml)
	for rel in src:gmatch('path%s*=%s*"%.%./([%w_]+)"') do
		deps[rel] = true
	end
	return deps
end

return {
	id = "repo.rust_crate",
	description = "Crates of the rust/ workspace (rust/crates/<name>/Cargo.toml)",
	identify = function(c)
		return c:matchFiles("rust/crates/{crate}/Cargo.toml")
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end

		local nameHits = c:query({
			path = path,
			language = "toml",
			query = "repo_toml.package_name",
			capture = "packageName",
		})
		local lintHits = c:query({
			path = path,
			language = "toml",
			query = "repo_toml.inherits_workspace_lints",
			capture = "enabled",
		})
		local inheritsLints = false
		for _, hit in ipairs(lintHits) do
			if hit.text == "true" then
				inheritsLints = true
			end
		end

		return {
			id = locator.crate,
			componentId = "repo.rust_crate",
			source = { path = path },
			dir = locator.crate,
			packageName = unquote(#nameHits > 0 and nameHits[1].text or nil),
			inheritsWorkspaceLints = inheritsLints,
			pathDeps = pathDeps(c, path),
		}
	end,
	verify = function(c, instance)
		local findings = {}
		local ast = instance.ast

		local memberHits = c:query({
			path = "rust/Cargo.toml",
			language = "toml",
			query = "repo_toml.workspace_members",
			capture = "member",
		})
		local want = "crates/" .. ast.dir
		local listed = false
		for _, hit in ipairs(memberHits) do
			if unquote(hit.text) == want then
				listed = true
				break
			end
		end
		if not listed then
			findings[#findings + 1] = {
				severity = "error",
				code = "crate.not_in_workspace_members",
				message = "rust/crates/" .. ast.dir .. " is not listed in rust/Cargo.toml workspace members",
				instanceId = instance.id,
			}
		end

		local expectedName = "codemod_recipe_" .. ast.dir
		if ast.packageName ~= expectedName then
			findings[#findings + 1] = {
				severity = "warning",
				code = "crate.package_name_convention",
				message = "package name "
					.. tostring(ast.packageName)
					.. " does not follow convention "
					.. expectedName,
				instanceId = instance.id,
			}
		end

		if not ast.inheritsWorkspaceLints then
			findings[#findings + 1] = {
				severity = "warning",
				code = "crate.missing_workspace_lints",
				message = "crate does not inherit workspace lints ([lints] workspace = true)",
				instanceId = instance.id,
			}
		end

		local allowed = ALLOWED_PATH_DEPS[ast.dir]
		if allowed then
			for dep in pairs(ast.pathDeps or {}) do
				if not allowed[dep] then
					findings[#findings + 1] = {
						severity = "error",
						code = "crate.forbidden_path_dep",
						message = "crate '"
							.. ast.dir
							.. "' must not path-depend on '"
							.. dep
							.. "' (see target architecture crate DAG)",
						instanceId = instance.id,
					}
				end
			end
		end

		return findings
	end,
	relate = function(_c, instance, index)
		local edges = {}
		local deps = instance.ast and instance.ast.pathDeps or {}
		for dep in pairs(deps) do
			local to = "repo.rust_crate/" .. dep
			if index:byId(to) then
				edges[#edges + 1] = { kind = "uses", to = to }
			end
		end
		return edges
	end,
	toSummary = function(_c, instance)
		local ast = instance.ast
		return {
			kind = "rust-crate",
			title = ast.packageName or ast.dir,
			id = ast.id,
			path = ast.source and ast.source.path or nil,
			facts = {
				ast.inheritsWorkspaceLints and "workspace lints" or "own lints",
			},
		}
	end,
}
