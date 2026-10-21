-- Variants reachable without an extension/MCP surface in this repo, with the reason.
-- Keep empty unless a HostCommand is intentionally host-internal / out-of-repo only.
local INTERNAL = {}

local createCommand = require("../../lib/create_command")

return {
	id = "repo.host_command",
	description = "One HostCommand protocol variant (child of repo.host_protocol)",
	parse = function(_c, context)
		local hints = context.hints or {}
		local source = context.source
		if not source or not source.path then
			return nil
		end
		return {
			id = hints.rename or hints.variant,
			componentId = "repo.host_command",
			source = source,
			variant = hints.variant,
			rename = hints.rename,
		}
	end,
	relate = function(_c, _instance, index)
		local edges = {
			{ kind = "lives_in", to = "repo.zone/command.adapters" },
		}
		local engine = index:byComponent("repo.engine_edit")
		if engine and #engine > 0 then
			edges[#edges + 1] = { kind = "uses", to = engine[1].id }
		end
		local reg = index:byComponent("repo.recipe_registry")
		if reg and #reg > 0 then
			edges[#edges + 1] = { kind = "uses", to = reg[1].id }
		end
		return edges
	end,
	verify = function(_c, instance, graph)
		-- Graph-aware completeness: a command with no incoming exposes/sends edge
		-- is unreachable from any surface in this repo. Only judge when the graph
		-- models surfaces at all (any exposes/sends edge exists) — otherwise the
		-- workspace simply doesn't include surface components.
		local surfacesModeled = false
		for _, edge in ipairs(graph:edges()) do
			if edge.kind == "exposes" or edge.kind == "sends" then
				surfacesModeled = true
				break
			end
		end
		if not surfacesModeled then
			return {}
		end
		for _, edge in ipairs(graph:incoming(instance.id)) do
			if edge.kind == "exposes" or edge.kind == "sends" then
				return {}
			end
		end
		local variant = instance.ast.variant
		if variant and INTERNAL[variant] then
			return {}
		end
		return {
			{
				severity = "warning",
				code = "host_command.no_surface",
				message = "HostCommand variant '"
					.. tostring(variant)
					.. "' is not exposed by any MCP tool or hostBridge method",
			},
		}
	end,
	operations = {
		createCommand = {
			description = "Add a HostCommand enum variant + dispatch stub + TS union member",
			args = {
				{ name = "name", required = true, help = "Command name (any case; normalized to camelCase)" },
				{ name = "ifExists", required = false, help = "skip (default) | error" },
			},
			apply = function(c, args, _target, graph)
				createCommand.applyHost(c, args, graph)
			end,
		},
	},
	toSummary = function(_c, instance)
		local ast = instance.ast
		return {
			kind = "host-command",
			title = ast.rename or ast.variant,
			id = ast.id,
			path = ast.source and ast.source.path or nil,
			facts = { "variant " .. tostring(ast.variant) },
		}
	end,
}
