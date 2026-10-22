--- Host Luau `c:` API surface: #[host_method] declarations ↔ generated stubs.

-- Globs / leaf files after handlers + delegate grain splits (Phase P/Q).
local HANDLER_GLOBS = {
	"rust/crates/host/src/host_api/handlers/**/*.rs",
	"rust/crates/host/src/host_api/lua_runtime.rs",
	"rust/crates/host/src/component/delegate/**/*.rs",
}
local IDENTIFY_ROOT = "rust/crates/host/src/host_api/handlers/mod.rs"
local STUB_LUAU = ".codemod/stubs/codemod-context.d.luau"
local SURFACE_TS = "vscode_extension/src/extension/language/hostApiSurface.ts"

local function handlerPaths(c)
	local seen, out = {}, {}
	for _, glob in ipairs(HANDLER_GLOBS) do
		for _, path in ipairs(c:listFiles(glob)) do
			if not seen[path] then
				seen[path] = true
				out[#out + 1] = path
			end
		end
	end
	table.sort(out)
	return out
end

local function collectMethodNames(c)
	local names, order = {}, {}
	for _, path in ipairs(handlerPaths(c)) do
		local ok, src = pcall(function()
			return c:read(path)
		end)
		if ok and src then
			for block in src:gmatch("#%[host_method%((.-)%)%]") do
				local name = block:match('name%s*=%s*"([%w_]+)"')
				if name and not names[name] then
					names[name] = { path = path }
					order[#order + 1] = name
				end
			end
			for name in src:gmatch("name:%s*\"([%w_]+)\"%s*,%s*\n%s*description:") do
				if not names[name] then
					names[name] = { path = path }
					order[#order + 1] = name
				end
			end
		end
	end
	table.sort(order)
	return names, order
end

local function stubMethods(c)
	local src = c:read(STUB_LUAU)
	local set = {}
	local ctx = src:match("export type CodemodContext = {(.-)\n}")
	if not ctx then
		return set, false
	end
	for name in ctx:gmatch("\n\t([%w_]+):") do
		set[name] = true
	end
	return set, true
end

local function tsSurfaceMethods(c)
	local src = c:read(SURFACE_TS)
	local set = {}
	for name in src:gmatch("name:%s*'([%w_]+)'") do
		set[name] = true
	end
	return set
end

return {
	id = "repo.host_api_method",
	description = "Luau c: host methods: #[host_method] ↔ codemod-context stub ↔ hostApiSurface.ts",
	identify = function(c)
		-- Singleton root keyed off handlers/mod.rs (primary #[host_method] tree).
		local out = {}
		for _, path in ipairs(c:listFiles(IDENTIFY_ROOT)) do
			out[#out + 1] = { path = path }
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local byName, order = collectMethodNames(c)
		local identity = {
			componentId = "repo.host_api_method",
			id = "host-api-surface",
			path = path,
		}
		-- Span map: function name → whole fn span across handler files.
		local spanByName = {}
		for _, hpath in ipairs(handlerPaths(c)) do
			local ok = pcall(function()
				local names = c:query({
					path = hpath,
					language = "rust",
					query = "repo_rust.function_items",
					capture = "fnName",
				})
				local fns = c:query({
					path = hpath,
					language = "rust",
					query = "repo_rust.function_items",
					capture = "fn",
				})
				table.sort(names, function(a, b)
					return a.span.from < b.span.from
				end)
				table.sort(fns, function(a, b)
					return a.span.from < b.span.from
				end)
				for i = 1, math.min(#names, #fns) do
					local fnName = names[i].text
					local spanned = { path = hpath, span = fns[i].span }
					spanByName[fnName] = spanned
					-- Luau c: names map to do_* Rust helpers (insert → do_insert).
					local short = fnName:match("^do_(.+)$")
					if short and not spanByName[short] then
						spanByName[short] = spanned
					end
				end
			end)
			if not ok then
				-- ignore missing/unparseable handler paths
			end
		end
		local children = {}
		local methods = {}
		for _, name in ipairs(order) do
			local site = byName[name]
			local spanned = spanByName[name]
			local source = { path = site.path }
			if spanned then
				source = { path = spanned.path, span = spanned.span }
			end
			methods[#methods + 1] = {
				id = name,
				name = name,
				path = site.path,
			}
			local child = c:component("repo.host_api_method_entry"):parse({
				source = source,
				parent = identity,
				role = "method",
				hints = { name = name },
			})
			if child then
				child.componentId = "repo.host_api_method_entry"
				child.role = "method"
				children[#children + 1] = child
			end
		end
		return {
			id = "host-api-surface",
			componentId = "repo.host_api_method",
			source = { path = path },
			methods = methods,
			methodOrder = order,
			children = children,
		}
	end,
	verify = function(c, instance)
		local findings = {}
		local stubs, stubOk = stubMethods(c)
		local ts = tsSurfaceMethods(c)
		if not stubOk then
			findings[#findings + 1] = {
				severity = "error",
				code = "host_api.stub_missing_context",
				message = "CodemodContext type not found in " .. STUB_LUAU,
				instanceId = instance.id,
			}
		end
		for _, name in ipairs(instance.ast.methodOrder or {}) do
			if stubOk and not stubs[name] then
				findings[#findings + 1] = {
					severity = "error",
					code = "host_api.missing_stub",
					message = "host method '"
						.. name
						.. "' is declared via #[host_method] but missing from "
						.. STUB_LUAU
						.. " (regenerate host API surface)",
					instanceId = instance.id,
				}
			end
			if not ts[name] then
				findings[#findings + 1] = {
					severity = "error",
					code = "host_api.missing_ts_surface",
					message = "host method '"
						.. name
						.. "' is declared via #[host_method] but missing from "
						.. SURFACE_TS,
					instanceId = instance.id,
				}
			end
		end
		-- Reverse: stub methods that have no host declaration (stale generation)
		if stubOk then
			local declared = {}
			for _, name in ipairs(instance.ast.methodOrder or {}) do
				declared[name] = true
			end
			for name in pairs(stubs) do
				if not declared[name] then
					findings[#findings + 1] = {
						severity = "warning",
						code = "host_api.stale_stub",
						message = "stub method '"
							.. name
							.. "' has no #[host_method] / HostMethodSurface declaration",
						instanceId = instance.id,
					}
				end
			end
		end
		return findings
	end,
	relate = function(_c, instance, index)
		local edges = {
			{ kind = "lives_in", to = "repo.zone/domain.host_api" },
		}
		local engine = index:byComponent("repo.engine_edit")
		if engine and #engine > 0 then
			edges[#edges + 1] = { kind = "drives", to = engine[1].id }
		end
		local tree = index:byComponent("repo.working_tree")
		if tree and #tree > 0 then
			edges[#edges + 1] = { kind = "stages_via", to = tree[1].id }
		end
		-- Bridges Luau QuerySpec → engine QuerySpec (D-016 converts).
		local engineQs = index:byId("repo.domain_type/QuerySpec.engine")
		if engineQs then
			edges[#edges + 1] = { kind = "converts", to = engineQs.id }
		end
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "host-api-surface",
			title = "Host API (c:)",
			id = instance.ast.id,
			facts = { #(instance.ast.methodOrder or {}) .. " methods" },
		}
	end,
}
