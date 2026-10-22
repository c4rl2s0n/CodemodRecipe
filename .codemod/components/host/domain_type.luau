--- Domain types: shared shapes live in core; Luau keeps FromLua adapters (D-016).

local SITES = {
	{
		id = "QuerySpec.core",
		name = "QuerySpec",
		path = "rust/crates/core/src/query_spec.rs",
		role = "owner",
	},
	{
		id = "QuerySpec.lua",
		name = "QuerySpec",
		path = "rust/crates/lua/src/host_api/specs/query.rs",
		role = "lua_adapter",
		pair = "QuerySpec.core",
	},
	{
		id = "IfExists.core",
		name = "IfExists",
		path = "rust/crates/core/src/apply/file_change.rs",
		role = "owner",
	},
	{
		id = "IfExists.lua",
		name = "IfExists",
		path = "rust/crates/lua/src/host_api/specs/common.rs",
		role = "lua_adapter",
		pair = "IfExists.core",
	},
}

return {
	id = "repo.domain_type",
	description = "Domain types: core ownership + Luau FromLua adapters (D-016)",
	identify = function(c)
		local out = {}
		for _, site in ipairs(SITES) do
			for _, path in ipairs(c:listFiles(site.path)) do
				out[#out + 1] = {
					path = path,
					hints = {
						siteId = site.id,
						typeName = site.name,
						pair = site.pair,
						role = site.role,
					},
				}
			end
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		local hints = locator.hints or {}
		local siteId = hints.siteId
		local typeName = hints.typeName
		if not path or not siteId or not typeName then
			return nil
		end
		local typeHits = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.type_items",
			capture = "typeItem",
		})
		local nameHits = c:query({
			path = path,
			language = "rust",
			query = "repo_rust.type_items",
			capture = "typeName",
		})
		local span = nil
		local found = false
		for i, nameHit in ipairs(nameHits) do
			if nameHit.text == typeName then
				found = true
				if typeHits[i] then
					span = typeHits[i].span
				end
				break
			end
		end
		local source = { path = path }
		if span then
			source.span = span
		end
		-- Full-file coverage leaves (domain_type is sole root for these paths).
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
				children[#children + 1] = {
					id = path .. ":" .. typeNames[i].text .. "@" .. tostring(types[i].span.from),
					componentId = "repo.host_module_item",
					source = { path = path, span = types[i].span },
					kind = "type",
					name = typeNames[i].text,
					role = "type",
				}
			end
		end)
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
				children[#children + 1] = {
					id = path .. ":" .. fnNames[i].text .. "@" .. tostring(fns[i].span.from),
					componentId = "repo.host_module_item",
					source = { path = path, span = fns[i].span },
					kind = "fn",
					name = fnNames[i].text,
					role = "fn",
				}
			end
		end)
		return {
			id = siteId,
			componentId = "repo.domain_type",
			source = source,
			typeName = typeName,
			pair = hints.pair,
			role = hints.role or "owner",
			defined = found,
			children = children,
		}
	end,
	relate = function(_c, instance, index)
		local edges = {
			{ kind = "lives_in", to = "repo.zone/domain.types" },
		}
		local pairId = instance.ast.pair
		if pairId then
			local target = "repo.domain_type/" .. pairId
			if index:byId(target) then
				edges[#edges + 1] = { kind = "mirrors", to = target }
			end
		end
		return edges
	end,
	verify = function(_c, instance)
		local findings = {}
		if not instance.ast.defined then
			findings[#findings + 1] = {
				severity = "error",
				code = "domain_type.missing",
				message = instance.ast.typeName .. " not found at " .. tostring(
					instance.ast.source and instance.ast.source.path
				),
				instanceId = instance.id,
			}
		end
		-- Luau adapters are expected (FromLua); only warn on unexpected dual owners.
		if instance.ast.role == "lua_adapter" then
			return findings
		end
		return findings
	end,
	toSummary = function(_c, instance)
		return {
			kind = "domain-type",
			title = instance.ast.typeName,
			id = instance.ast.id,
			facts = { instance.ast.source and instance.ast.source.path or "" },
		}
	end,
}
