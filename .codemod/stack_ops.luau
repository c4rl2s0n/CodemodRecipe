--- Stack-level operations for repo-stack (thin fan-out to layer components).
--- Library module: no top-level id/apply/parse.

local createCommand = require("./lib/create_command")

local VALID_LAYERS = {
	host = "repo.host_command",
	mcp = "repo.mcp_tool",
	vscode = "repo.vscode_command",
}

local function normalizeLayers(layers)
	if layers == nil then
		return { "host", "mcp", "vscode" }
	end
	local out = {}
	if type(layers) == "string" then
		out[1] = layers
	elseif type(layers) == "table" then
		for _, layer in ipairs(layers) do
			out[#out + 1] = layer
		end
	else
		error("layers must be a string or array of strings")
	end
	if #out == 0 then
		error("layers must not be empty")
	end
	return out
end

return {
	operations = {
		createCommand = {
			description = "Fan out createCommand to selected layers (host / mcp / vscode)",
			args = {
				{ name = "name", required = true, help = "Command name (normalized across layers)" },
				{
					name = "layers",
					required = false,
					help = "Subset of host|mcp|vscode (default: all three)",
				},
				{ name = "ifExists", required = false, help = "skip (default) | error" },
			},
			apply = function(c, args, graph)
				if not args.name or args.name == "" then
					error("createCommand requires args.name")
				end
				local forms = createCommand.forms(args.name)
				-- Cross-layer guard: name taken anywhere → skip/error once.
				local taken = createCommand.hostHasCommand(graph, forms.rename)
					or createCommand.mcpHasTool(graph, forms.mcpTool)
					or createCommand.vscodeHasCommand(graph, forms.commandId)
				if taken and args.ifExists == "error" then
					error("command '" .. forms.rename .. "' already exists in at least one layer")
				end

				for _, layer in ipairs(normalizeLayers(args.layers)) do
					local componentId = VALID_LAYERS[layer]
					if not componentId then
						error("unknown layer '" .. tostring(layer) .. "' (expected host|mcp|vscode)")
					end
					c:component(componentId):operation("createCommand", {
						name = args.name,
						ifExists = args.ifExists,
					})
				end
			end,
		},
	},
}
