--- Catalogued architecture / structural zones from stack.zones (via lib/zones).

local catalog = require("../../lib/zones")

return {
	id = "repo.zone",
	description = "Named membership zones (subgraph containers) from the stack catalog",
	identify = function(_c)
		local out = {}
		for _, zone in ipairs(catalog.zones or {}) do
			out[#out + 1] = {
				id = zone.id,
				role = zone.role,
				title = zone.title,
				parentId = zone.parentId,
			}
		end
		return out
	end,
	parse = function(_c, locator)
		local id = locator.id
		if not id then
			return nil
		end
		return {
			id = id,
			componentId = "repo.zone",
			role = locator.role,
			title = locator.title,
			parentId = locator.parentId,
		}
	end,
	relate = function(_c, instance, _index)
		local parentId = instance.ast.parentId
		if type(parentId) ~= "string" or parentId == "" then
			return {}
		end
		return {
			{ kind = "lives_in", to = "repo.zone/" .. parentId },
		}
	end,
	toSummary = function(_c, instance)
		local role = instance.ast.role or "zone"
		local title = instance.ast.title or instance.ast.id
		return {
			kind = "zone",
			title = title,
			id = instance.ast.id,
			facts = { "role=" .. role },
		}
	end,
}
