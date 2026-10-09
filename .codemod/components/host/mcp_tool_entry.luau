return {
	id = "repo.mcp_tool_entry",
	description = "One MCP tool (child of repo.mcp_tool)",
	parse = function(_c, context)
		local hints = context.hints or {}
		if not hints.name then
			return nil
		end
		return {
			id = hints.name,
			componentId = "repo.mcp_tool_entry",
			source = context.source,
			-- HostCommand variant names this tool routes to (empty for direct handlers)
			variants = hints.variants or {},
			-- true when the arm legitimately bypasses HostCommand (e.g. bootstrap_project)
			direct = hints.direct or false,
			-- helper fn name when routed indirectly (e.g. mcp_preview_or_apply)
			helper = hints.helper,
			-- present in the tools/list inventory
			listed = hints.listed or false,
		}
	end,
	relate = function(_c, instance, index)
		local ast = instance.ast
		local edges = {}
		for _, hc in ipairs(index:byComponent("repo.host_command")) do
			for _, variant in ipairs(ast.variants or {}) do
				if hc.ast.variant == variant then
					edges[#edges + 1] = {
						kind = "exposes",
						to = hc.id,
						source = ast.source,
					}
				end
			end
		end
		if #edges == 0 then
			return nil
		end
		return edges
	end,
	toSummary = function(_c, instance)
		local ast = instance.ast
		local target = ast.direct and "direct handler" or table.concat(ast.variants or {}, ", ")
		return {
			kind = "mcp-tool",
			title = ast.id,
			id = ast.id,
			path = ast.source and ast.source.path or nil,
			facts = { "routes to " .. target },
		}
	end,
}
