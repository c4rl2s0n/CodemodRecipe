local MCP_RS = "rust/crates/host/src/bin/codemod_mcp.rs"
local INVENTORY_RS = "rust/crates/host/src/protocol/mcp_inventory.rs"
local COMMAND_RS = "rust/crates/host/src/protocol/command.rs"
local DISPATCH_RS = "rust/crates/host/src/protocol/dispatch/mod.rs"
local HOST_COMMAND_TS = "vscode_extension/src/shared/hostCommand.ts"

--- Minimal MCP binary: tools/list derives from inventory; tools/call routes tools.
local function mcpRs(callArms, helpers)
	return [[
fn handle_request(registry: &mut RecipeRegistry, request: &RpcRequest) -> RpcResponse {
    match request.method.as_str() {
        "initialize" => RpcResponse { jsonrpc: "2.0" },
        "tools/list" => RpcResponse {
            result: Some(json!({
              "tools": tools_array(),
            })),
        },
        "tools/call" => {
            let tool = request.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let result = match tool {
]]
		.. callArms
		.. [[
                _ => error_json(format!("Unknown tool: {tool}")),
            };
            RpcResponse { jsonrpc: "2.0" }
        }
        _ => RpcResponse { jsonrpc: "2.0" },
    }
}

]]
		.. (helpers or "")
end

local function inventoryRs(toolMetas)
	return [[
pub struct McpToolMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub host_command: Option<&'static str>,
}
pub fn tool_metas() -> &'static [McpToolMeta] {
    &[
]]
		.. toolMetas
		.. [[
    ]
}
pub fn tools_array() -> Vec<serde_json::Value> { vec![] }
]]
end

local function meta(name, variant)
	if variant then
		return '        McpToolMeta { name: "'
			.. name
			.. '", description: "doc", host_command: Some("'
			.. variant
			.. '"), },\n'
	end
	return '        McpToolMeta { name: "' .. name .. '", description: "doc", host_command: None, },\n'
end

local function unitArm(tool, variant)
	return '                "' .. tool .. '" => handle_command(registry, HostCommand::' .. variant .. '),\n'
end

local function structArm(tool, variant)
	return '                "'
		.. tool
		.. '" => handle_command(registry, HostCommand::'
		.. variant
		.. ' {\n                    recipe: recipe_id.to_string(),\n                }),\n'
end

local previewHelpers = [[
fn mcp_preview_or_apply(registry: &mut RecipeRegistry, arguments: &serde_json::Value, do_apply: bool) -> serde_json::Value {
    if do_apply {
        handle_command(registry, HostCommand::Apply { recipe, args, preview_token, selection })
    } else {
        handle_command(registry, HostCommand::Preview { recipe, args, snippet_lines })
    }
}
]]

local BASE_METAS = meta("list_recipes", "List")
	.. meta("describe_recipe", "Describe")
	.. meta("preview_recipe", "Preview")
	.. meta("apply_recipe", "Apply")
	.. meta("bootstrap_project", nil)

local BASE_CALL = unitArm("list_recipes", "List")
	.. structArm("describe_recipe", "Describe")
	.. '                "preview_recipe" => mcp_preview_or_apply(registry, &arguments, false),\n'
	.. '                "apply_recipe" => mcp_preview_or_apply(registry, &arguments, true),\n'
	.. '                "bootstrap_project" => bootstrap::bootstrap_project(&registry.workspace_root),\n'

local function baseFiles(overrides)
	local files = {
		[MCP_RS] = mcpRs(BASE_CALL, previewHelpers),
		[INVENTORY_RS] = inventoryRs(BASE_METAS),
		[COMMAND_RS] = [[
#[derive(Debug, Deserialize)]
#[serde(tag = "command")]
pub enum HostCommand {
    #[serde(rename = "list")]
    List,
    #[serde(rename = "describe")]
    Describe { recipe: String },
    #[serde(rename = "preview")]
    Preview { recipe: Option<String> },
    #[serde(rename = "apply")]
    Apply { recipe: Option<String> },
}
]],
		[DISPATCH_RS] = [[
pub fn handle_command(cmd: HostCommand) {
    match cmd {
        HostCommand::List => list(),
        HostCommand::Describe { recipe } => describe(recipe),
        HostCommand::Preview { .. } => preview(),
        HostCommand::Apply { .. } => apply(),
    }
}
]],
		[HOST_COMMAND_TS] = [[
export type HostCommand =
  | { command: 'list' }
  | { command: 'describe'; recipe: string }
  | { command: 'preview'; recipe?: string }
  | { command: 'apply'; recipe?: string };
]],
	}
	for k, v in pairs(overrides or {}) do
		files[k] = v
	end
	return files
end

local function codes(findings)
	local out = {}
	for _, f in ipairs(findings) do
		out[#out + 1] = f.code .. "@" .. tostring(f.instanceId)
	end
	table.sort(out)
	return out
end

--- Full stack emits composer.* noise in sparse fixtures; assert MCP-family only.
local function mcpFindings(findings)
	local out = {}
	for _, f in ipairs(findings or {}) do
		local code = f.code or ""
		if code:find("^mcp_tool%.", 1) then
			out[#out + 1] = f
		end
	end
	return out
end

return {
	id = "repo.mcp_tool.tests",
	target = "repo.mcp_tool",
	tests = {
		consistent_surface_has_exposes_edges_and_no_findings = function(t)
			local ws = t:workspace({ files = baseFiles() })
			local scan = ws:scan()
			t:assertEq(#mcpFindings(scan.findings), 0)
			local root
			for _, inst in ipairs(scan.instances) do
				if inst.componentId == "repo.mcp_tool" then
					root = inst
				end
			end
			t:assert(root, "mcp_tool root missing")
			local toolChildren = 0
			for _, child in ipairs(root.ast.children or {}) do
				if child.componentId == "repo.mcp_tool_entry" then
					toolChildren += 1
				end
			end
			t:assertEq(toolChildren, 5)
			local edges = {}
			for _, e in ipairs(scan.edges or {}) do
				edges[e.from .. "->" .. (e.to or e.symbol or "")] = e.kind
			end
			t:assertEq(edges["repo.mcp_tool_entry/list_recipes->repo.host_command/list"], "exposes")
			t:assertEq(edges["repo.mcp_tool_entry/describe_recipe->repo.host_command/describe"], "exposes")
			t:assertEq(edges["repo.mcp_tool_entry/preview_recipe->repo.host_command/preview"], "exposes")
			t:assertEq(edges["repo.mcp_tool_entry/apply_recipe->repo.host_command/apply"], "exposes")
			t:assertEq(edges["repo.mcp_tool_entry/bootstrap_project->repo.host_command/list"], nil)
		end,

		listed_tool_without_arm_is_flagged = function(t)
			local ws = t:workspace({
				files = baseFiles({
					[INVENTORY_RS] = inventoryRs(BASE_METAS .. meta("ghost_tool", "List")),
				}),
			})
			local scan = ws:scan()
			t:assertEq(codes(mcpFindings(scan.findings)), { "mcp_tool.missing_arm@repo.mcp_tool/mcp-surface" })
		end,

		arm_without_listing_is_flagged = function(t)
			local ws = t:workspace({
				files = baseFiles({
					[MCP_RS] = mcpRs(BASE_CALL .. unitArm("run_tests", "RunTests"), previewHelpers),
				}),
			})
			local scan = ws:scan()
			t:assertEq(codes(mcpFindings(scan.findings)), {
				"mcp_tool.unknown_variant@repo.mcp_tool_entry/run_tests",
				"mcp_tool.unlisted@repo.mcp_tool_entry/run_tests",
			})
		end,

		helper_body_drift_is_flagged = function(t)
			local ws = t:workspace({
				files = baseFiles({
					[MCP_RS] = mcpRs(
						BASE_CALL,
						[[
fn mcp_preview_or_apply(registry: &mut RecipeRegistry, arguments: &serde_json::Value, do_apply: bool) -> serde_json::Value {
    handle_command(registry, HostCommand::Apply { recipe, args, preview_token, selection })
}
]]
					),
				}),
			})
			local scan = ws:scan()
			t:assertEq(codes(mcpFindings(scan.findings)), { "mcp_tool.helper_drift@repo.mcp_tool_entry/preview_recipe" })
		end,

		decoy_scoped_identifiers_are_not_host_command_refs = function(t)
			local ws = t:workspace({
				files = baseFiles({
					[MCP_RS] = mcpRs(
						BASE_CALL
							.. '                "run_tests" => { let _x = protocol_keys::ERROR; handle_command(registry, HostCommand::List) }\n',
						previewHelpers
					),
				}),
			})
			local scan = ws:scan()
			t:assertEq(codes(mcpFindings(scan.findings)), { "mcp_tool.unlisted@repo.mcp_tool_entry/run_tests" })
		end,

		handwritten_tools_list_is_flagged = function(t)
			local ws = t:workspace({
				files = baseFiles({
					[MCP_RS] = [[
fn handle_request(registry: &mut RecipeRegistry, request: &RpcRequest) -> RpcResponse {
    match request.method.as_str() {
        "tools/list" => RpcResponse {
            result: Some(json!({
              "tools": [
                { "name": "list_recipes", "description": "doc", "inputSchema": { "type": "object" } },
              ]
            })),
        },
        "tools/call" => {
            let tool = request.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let result = match tool {
                "list_recipes" => handle_command(registry, HostCommand::List),
                "describe_recipe" => handle_command(registry, HostCommand::Describe { recipe: recipe_id.to_string() }),
                "preview_recipe" => mcp_preview_or_apply(registry, &arguments, false),
                "apply_recipe" => mcp_preview_or_apply(registry, &arguments, true),
                "bootstrap_project" => bootstrap::bootstrap_project(&registry.workspace_root),
                _ => error_json(format!("Unknown tool: {tool}")),
            };
            RpcResponse { jsonrpc: "2.0" }
        }
        _ => RpcResponse { jsonrpc: "2.0" },
    }
}
]] .. previewHelpers,
				}),
			})
			local scan = ws:scan()
			local found = false
			for _, f in ipairs(scan.findings) do
				if f.code == "mcp_tool.handwritten_list" then
					found = true
				end
			end
			t:assert(found, "expected mcp_tool.handwritten_list")
		end,
	},
}
