--- Fixture-level tests for stack/layer createCommand (uses the real component modules
--- under a temp workspace with minimal protocol/mcp/vscode surfaces).

local MOD_RS = "rust/crates/host/src/protocol/mod.rs"
local DISPATCH_RS = "rust/crates/host/src/protocol/dispatch/mod.rs"
local TYPES_TS = "vscode_extension/src/shared/types.ts"
local MCP_RS = "rust/crates/host/src/bin/codemod_mcp.rs"
local INVENTORY_RS = "rust/crates/host/src/protocol/mcp_inventory.rs"
local PACKAGE_JSON = "vscode_extension/package.json"
local CONSTANTS_TS = "vscode_extension/src/extension/constants.ts"
local EXTENSION_TS = "vscode_extension/src/extension/extension.ts"

local function baseFiles()
	return {
		[MOD_RS] = [[
#[derive(Debug, Deserialize)]
#[serde(tag = "command")]
pub enum HostCommand {
    #[serde(rename = "list")]
    List,
    #[serde(rename = "runTests")]
    RunTests {
        #[serde(default)]
        filter: Option<String>,
    },
}
]],
		[DISPATCH_RS] = [[
pub fn handle_command(cmd: HostCommand) -> serde_json::Value {
    match cmd {
        HostCommand::List => serde_json::json!({"ok": true}),
        HostCommand::RunTests { filter } => {
            let _ = filter;
            serde_json::json!({"ok": true})
        }
    }
}
]],
		[TYPES_TS] = [[
export type HostCommand =
  | { command: 'list' }
  | { command: 'runTests'; filter?: string };
]],
		[INVENTORY_RS] = [[
pub struct McpToolMeta {
    pub name: &'static str,
    pub description: &'static str,
    pub host_command: Option<&'static str>,
}
pub fn tool_metas() -> &'static [McpToolMeta] {
    &[
        McpToolMeta { name: "list_recipes", description: "doc", host_command: Some("List"), },
        McpToolMeta { name: "run_tests", description: "doc", host_command: Some("RunTests"), },
    ]
}
fn input_schema(name: &str) -> serde_json::Value {
    match name {
        "list_recipes" => serde_json::json!({ "type": "object" }),
        "run_tests" => serde_json::json!({ "type": "object" }),
        _ => serde_json::json!({ "type": "object" }),
    }
}
pub fn tools_array() -> Vec<serde_json::Value> { vec![] }
]],
		[MCP_RS] = [[
fn handle_request(registry: &mut RecipeRegistry, request: &RpcRequest) -> RpcResponse {
    match request.method.as_str() {
        "tools/list" => RpcResponse {
            result: Some(json!({
              "tools": tools_array(),
            })),
        },
        "tools/call" => {
            let tool = request.params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let result = match tool {
                "list_recipes" => handle_command(registry, HostCommand::List),
                "run_tests" => handle_command(registry, HostCommand::RunTests { filter: None }),
                _ => error_json(format!("Unknown tool: {tool}")),
            };
            RpcResponse { jsonrpc: "2.0" }
        }
        _ => RpcResponse { jsonrpc: "2.0" },
    }
}
]],
		[PACKAGE_JSON] = [[
{
  "name": "codemod-recipe",
  "contributes": {
    "commands": [
      { "command": "codemodRecipe.refresh", "title": "Refresh" }
    ]
  }
}
]],
		[CONSTANTS_TS] = [[
export const COMMANDS = {
  refresh: 'codemodRecipe.refresh',
} as const;
]],
		[EXTENSION_TS] = [[
import * as vscode from 'vscode';
export function activate(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand(COMMANDS.refresh, () => refresh()),
  );
}
]],
		["vscode_extension/src/extension/host/hostBridge.ts"] = [[
export class HostBridge {
  async listRecipes(): Promise<void> {
    await this.send<void>({ command: 'list' });
  }
  private async send<T>(command: HostCommand): Promise<T> {
    return Promise.resolve({} as T);
  }
}
]],
	}
end

return {
	id = "repo.create_command.tests",
	target = "repo-stack",
	tests = {
		stack_create_command_fans_out_to_host_and_mcp = function(t)
			local ws = t:workspace({ files = baseFiles() })
			ws:stackOperation("createCommand", {
				name = "componentSnapshot",
				layers = { "host", "mcp" },
			})
			local mod = ws:readFile(MOD_RS)
			t:assert(mod:find("ComponentSnapshot"), "enum variant missing")
			t:assert(mod:find('rename = "componentSnapshot"'), "serde rename missing")
			local mcp = ws:readFile(MCP_RS)
			t:assert(mcp:find('"component_snapshot" =>'), "tools/call arm missing")
			local inv = ws:readFile(INVENTORY_RS)
			t:assert(inv:find('name: "component_snapshot"'), "inventory entry missing")
			local types = ws:readFile(TYPES_TS)
			t:assert(types:find("componentSnapshot"), "TS union member missing")
			-- vscode layer not selected
			local pkg = ws:readFile(PACKAGE_JSON)
			t:assert(not pkg:find("componentSnapshot"), "vscode should be untouched")
		end,

		create_command_skips_when_present = function(t)
			local ws = t:workspace({ files = baseFiles() })
			ws:stackOperation("createCommand", {
				name = "list",
				layers = { "host" },
				ifExists = "skip",
			})
			-- still just List + RunTests
			local mod = ws:readFile(MOD_RS)
			local _, count = mod:gsub("serde%(rename", "")
			t:assertEq(count, 2)
		end,
	},
}
