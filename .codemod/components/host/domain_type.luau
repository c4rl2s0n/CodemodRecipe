--- Dual-owned domain types (QuerySpec / IfExists) — architecture smell D-016.

local SITES = {
	{
		id = "QuerySpec.engine",
		name = "QuerySpec",
		path = "rust/crates/engine/src/ops.rs",
		pair = "QuerySpec.lua",
	},
	{
		id = "QuerySpec.lua",
		name = "QuerySpec",
		path = "rust/crates/lua/src/host_api/specs.rs",
		pair = "QuerySpec.engine",
	},
	{
		id = "IfExists.core",
		name = "IfExists",
		path = "rust/crates/core/src/apply/file_change.rs",
		pair = "IfExists.lua",
	},
	{
		id = "IfExists.lua",
		name = "IfExists",
		path = "rust/crates/lua/src/host_api/specs.rs",
		pair = "IfExists.core",
	},
}

return {
	id = "repo.domain_type",
	description = "Domain types with dual definitions (QuerySpec, IfExists) — D-016",
	identify = function(c)
		local out = {}
		for _, site in ipairs(SITES) do
			for _, path in ipairs(c:listFiles(site.path)) do
				out[#out + 1] = {
					path = path,
					hints = { siteId = site.id, typeName = site.name, pair = site.pair },
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
		return {
			id = siteId,
			componentId = "repo.domain_type",
			source = source,
			typeName = typeName,
			pair = hints.pair,
			defined = found,
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
		-- Architecture-first: dual ownership is intentional debt until unified.
		findings[#findings + 1] = {
			severity = "warning",
			code = "domain_type.dual_ownership",
			message = instance.ast.typeName
				.. " is defined in multiple crates (D-016); unify or keep explicit bridge",
			instanceId = instance.id,
		}
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
