local PACKAGE_JSON = "vscode_extension/package.json"
local CONSTANTS_TS = "vscode_extension/src/extension/constants.ts"
local EXTENSION_TS = "vscode_extension/src/extension/extension.ts"

local function packageJson()
	return [[
{
  "name": "codemod-recipe",
  "contributes": {
    "commands": [
      { "command": "codemodRecipe.refresh", "title": "Refresh" },
      { "command": "codemodRecipe.validateRecipes", "title": "Validate" }
    ]
  }
}
]]
end

local function constantsTs()
	return [[
export const COMMANDS = {
  refresh: 'codemodRecipe.refresh',
  validateRecipes: 'codemodRecipe.validateRecipes',
} as const;
]]
end

local function extensionTs()
	return [[
import * as vscode from 'vscode';
export function activate(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand(COMMANDS.refresh, () => refresh()),
    vscode.commands.registerCommand(COMMANDS.validateRecipes, () => validate()),
  );
}
]]
end

local function codes(findings)
	local out = {}
	for _, f in ipairs(findings) do
		out[#out + 1] = f.code .. "@" .. tostring(f.instanceId)
	end
	table.sort(out)
	return out
end

local function vscodeRoot(scan)
	for _, inst in ipairs(scan.instances or {}) do
		if inst.componentId == "repo.vscode_command" then
			return inst
		end
	end
	return nil
end

local function vscodeFindings(findings)
	local out = {}
	for _, f in ipairs(findings or {}) do
		if (f.code or ""):find("^vscode_command%.", 1) then
			out[#out + 1] = f
		end
	end
	return out
end

return {
	id = "repo.vscode_command.tests",
	target = "repo.vscode_command",
	tests = {
		consistent_wiring_has_no_findings = function(t)
			local ws = t:workspace({
				files = {
					[PACKAGE_JSON] = packageJson(),
					[CONSTANTS_TS] = constantsTs(),
					[EXTENSION_TS] = extensionTs(),
				},
			})
			local scan = ws:scan()
			local root = vscodeRoot(scan)
			t:assert(root ~= nil)
			t:assertEq(root.componentId, "repo.vscode_command")
			t:assertEq(#root.ast.children, 2)
			t:assertEq(#vscodeFindings(scan.findings), 0)
		end,

		verify_flags_manifest_command_without_constant = function(t)
			local ws = t:workspace({
				files = {
					[PACKAGE_JSON] = packageJson(),
					[CONSTANTS_TS] = "export const COMMANDS = {\n  refresh: 'codemodRecipe.refresh',\n} as const;\n",
					[EXTENSION_TS] = extensionTs(),
				},
			})
			local scan = ws:scan()
			t:assertEq(codes(vscodeFindings(scan.findings)), {
				"vscode_command.missing_constant@repo.vscode_command_entry/codemodRecipe.validateRecipes",
			})
		end,

		verify_flags_constant_without_manifest = function(t)
			local ws = t:workspace({
				files = {
					[PACKAGE_JSON] = packageJson(),
					[CONSTANTS_TS] = "export const COMMANDS = {\n  refresh: 'codemodRecipe.refresh',\n  validateRecipes: 'codemodRecipe.validateRecipes',\n  internal: 'codemodRecipe.internal',\n} as const;\n",
					[EXTENSION_TS] = extensionTs(),
				},
			})
			local scan = ws:scan()
			t:assertEq(codes(vscodeFindings(scan.findings)), {
				"vscode_command.dead_constant@repo.vscode_command_entry/codemodRecipe.internal",
				"vscode_command.hidden_command@repo.vscode_command_entry/codemodRecipe.internal",
			})
		end,

		verify_flags_literal_registration = function(t)
			local ws = t:workspace({
				files = {
					[PACKAGE_JSON] = packageJson(),
					[CONSTANTS_TS] = constantsTs(),
					[EXTENSION_TS] = "import * as vscode from 'vscode';\nexport function activate(context: vscode.ExtensionContext) {\n  vscode.commands.registerCommand(COMMANDS.refresh, () => refresh());\n  vscode.commands.registerCommand(COMMANDS.validateRecipes, () => validate());\n  vscode.commands.registerCommand('codemodRecipe.sneaky', () => sneaky());\n}\n",
				},
			})
			local scan = ws:scan()
			t:assertEq(codes(vscodeFindings(scan.findings)), {
				"vscode_command.literal_registration@repo.vscode_command/vscode-commands",
			})
		end,

		verify_flags_missing_registration = function(t)
			local ws = t:workspace({
				files = {
					[PACKAGE_JSON] = packageJson(),
					[CONSTANTS_TS] = constantsTs(),
					[EXTENSION_TS] = "import * as vscode from 'vscode';\nexport function activate(context: vscode.ExtensionContext) {\n  vscode.commands.registerCommand(COMMANDS.refresh, () => refresh());\n}\n",
				},
			})
			local scan = ws:scan()
			-- both perspectives flag the same gap: manifest entry unregistered,
			-- constant never passed to registerCommand
			t:assertEq(codes(vscodeFindings(scan.findings)), {
				"vscode_command.dead_constant@repo.vscode_command_entry/codemodRecipe.validateRecipes",
				"vscode_command.missing_registration@repo.vscode_command_entry/codemodRecipe.validateRecipes",
			})
		end,

		-- handler bodies calling bridge methods produce invokes edges
		handler_bridge_calls_resolve_to_invokes_edges = function(t)
			local ws = t:workspace({
				files = {
					[PACKAGE_JSON] = packageJson(),
					[CONSTANTS_TS] = constantsTs(),
					[EXTENSION_TS] = [[
import * as vscode from 'vscode';
const bridge = new HostBridge();
export function activate(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand(COMMANDS.refresh, () => bridge.listRecipes()),
    vscode.commands.registerCommand(COMMANDS.validateRecipes, () => validate()),
  );
}
]],
					["vscode_extension/src/extension/host/hostBridge.ts"] = [[
export class HostBridge {
  async listRecipes(): Promise<void> {}
  async ensureHost(): Promise<void> {}
}
]],
				},
			})
			local scan = ws:scan()
			t:assertEq(#vscodeFindings(scan.findings), 0)
			local edges = {}
			for _, e in ipairs(scan.edges or {}) do
				edges[e.from .. "->" .. e.to] = e.kind
			end
			t:assertEq(
				edges["repo.vscode_command_entry/codemodRecipe.refresh->repo.host_bridge_method/listRecipes"],
				"invokes"
			)
			-- module-level validate() is unresolvable: no edge, no finding
			t:assertEq(
				edges["repo.vscode_command_entry/codemodRecipe.validateRecipes->repo.host_bridge_method/listRecipes"],
				nil
			)
		end,

		-- a registered handler with an empty inline body is suspicious
		verify_flags_empty_handler = function(t)
			local ws = t:workspace({
				files = {
					[PACKAGE_JSON] = packageJson(),
					[CONSTANTS_TS] = constantsTs(),
					[EXTENSION_TS] = [[
import * as vscode from 'vscode';
export function activate(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand(COMMANDS.refresh, () => refresh()),
    vscode.commands.registerCommand(COMMANDS.validateRecipes, () => {}),
  );
}
]],
				},
			})
			local scan = ws:scan()
			t:assertEq(codes(vscodeFindings(scan.findings)), {
				"vscode_command.empty_handler@repo.vscode_command_entry/codemodRecipe.validateRecipes",
			})
		end,

		-- handler -> this.* controller method -> this.bridge.* (one hop, same file)
		controller_mediated_calls_resolve_one_hop = function(t)
			local ws = t:workspace({
				files = {
					[PACKAGE_JSON] = packageJson(),
					[CONSTANTS_TS] = constantsTs(),
					[EXTENSION_TS] = [[
import * as vscode from 'vscode';
class Controller {
  register(context: vscode.ExtensionContext) {
    context.subscriptions.push(
      vscode.commands.registerCommand(COMMANDS.refresh, async () => this.runRefresh()),
      vscode.commands.registerCommand(COMMANDS.validateRecipes, async () => this.runValidate()),
    );
  }
  private async runRefresh() {
    await this.bridge.listRecipes();
  }
  private async runValidate() {
    vscode.window.showInformationMessage('done');
  }
}
]],
					["vscode_extension/src/extension/host/hostBridge.ts"] = [[
export class HostBridge {
  async listRecipes(): Promise<void> {}
}
]],
				},
			})
			local scan = ws:scan()
			t:assertEq(#vscodeFindings(scan.findings), 0)
			local edges = {}
			for _, e in ipairs(scan.edges or {}) do
				edges[e.from .. "->" .. e.to] = e.kind
			end
			t:assertEq(
				edges["repo.vscode_command_entry/codemodRecipe.refresh->repo.host_bridge_method/listRecipes"],
				"invokes"
			)
			-- pure-UI hop: no bridge call, no edge, no finding
			local validateEdges = 0
			for _, e in ipairs(scan.edges or {}) do
				if e.from == "repo.vscode_command_entry/codemodRecipe.validateRecipes" then
					validateEdges += 1
				end
			end
			t:assertEq(validateEdges, 0)
		end,
	},
}
