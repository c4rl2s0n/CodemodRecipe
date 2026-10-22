--- Host API handler capability modules (post D-025 A2).

local CAPABILITIES = {
	{ id = "edits", path = "rust/crates/host/src/host_api/handlers/edits.rs" },
	{ id = "queries", path = "rust/crates/host/src/host_api/handlers/queries.rs" },
	{ id = "files", path = "rust/crates/host/src/host_api/handlers/files.rs" },
	{ id = "template_methods", path = "rust/crates/host/src/host_api/handlers/template_methods.rs" },
	{ id = "log", path = "rust/crates/host/src/host_api/handlers/log.rs" },
	{ id = "helpers", path = "rust/crates/host/src/host_api/handlers/helpers.rs" },
}

return {
	id = "repo.host_api_handlers",
	description = "host_api/handlers capability modules (insert/query/files/…)",
	identify = function(c)
		local out = {}
		for _, cap in ipairs(CAPABILITIES) do
			for _, path in ipairs(c:listFiles(cap.path)) do
				out[#out + 1] = { path = path, hints = { capability = cap.id } }
			end
		end
		return out
	end,
	parse = function(_c, locator)
		local path = locator.path
		local capability = locator.hints and locator.hints.capability
		if not path or not capability then
			return nil
		end
		return {
			id = capability,
			componentId = "repo.host_api_handlers",
			source = { path = path },
			capability = capability,
		}
	end,
	relate = function(_c, _instance)
		return {
			{ kind = "lives_in", to = "repo.zone/domain.host_api" },
			{ kind = "drives", to = "repo.engine_edit/engine-edit" },
			{ kind = "stages_via", to = "repo.working_tree/working-tree" },
		}
	end,
	verify = function(_c, _instance)
		return {}
	end,
	toSummary = function(_c, instance)
		return {
			kind = "host-api-handlers",
			title = instance.ast.capability or instance.ast.id,
			id = instance.ast.id,
			facts = { instance.ast.source and instance.ast.source.path or "" },
		}
	end,
}
