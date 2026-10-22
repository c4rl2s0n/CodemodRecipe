local BRIDGE_TS = "vscode_extension/src/extension/host/hostBridge.ts"
local COMMAND_RS = "rust/crates/host/src/protocol/command.rs"
local DISPATCH_RS = "rust/crates/host/src/protocol/dispatch/mod.rs"
local HOST_COMMAND_TS = "vscode_extension/src/shared/hostCommand.ts"

local function bridgeTs(extraMethod)
	return [[
export class HostBridge {
  async listRecipes(): Promise<RecipeCatalogResponse> {
    const response = await this.send<RecipeCatalogResponse>({ command: 'list' });
    return response;
  }

  async applyRecipe(recipe: string): Promise<ApplyResponse> {
    return this.sendPersistent<ApplyResponse>({
      command: 'apply',
      recipe,
    });
  }

]]
		.. (extraMethod or "")
		.. [[
  private async send<T>(command: HostCommand): Promise<T> {
    return this.sendOneShot<T>(command);
  }

  private async sendOneShot<T>(command: HostCommand): Promise<T> {
    return Promise.resolve({} as T);
  }
}
]]
end

local function baseFiles(overrides)
	local files = {
		[BRIDGE_TS] = bridgeTs(),
		[COMMAND_RS] = [[
#[derive(Debug, Deserialize)]
#[serde(tag = "command")]
pub enum HostCommand {
    #[serde(rename = "list")]
    List,
    #[serde(rename = "apply")]
    Apply { recipe: String },
}
]],
		[DISPATCH_RS] = [[
pub fn handle_command(cmd: HostCommand) {
    match cmd {
        HostCommand::List => list(),
        HostCommand::Apply { recipe } => apply(recipe),
    }
}
]],
		[HOST_COMMAND_TS] = [[
export type HostCommand =
  | { command: 'list' }
  | { command: 'apply'; recipe: string };
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

--- Full stack emits composer.* noise in sparse fixtures; assert bridge/relation only.
local function bridgeFindings(findings)
	local out = {}
	for _, f in ipairs(findings or {}) do
		local code = f.code or ""
		if code:find("^relation%.", 1) or code:find("^host_bridge", 1) then
			out[#out + 1] = f
		end
	end
	return out
end

return {
	id = "repo.host_bridge.tests",
	target = "repo.host_bridge",
	tests = {
		methods_with_literals_become_children_with_sends_edges = function(t)
			local ws = t:workspace({ files = baseFiles() })
			local scan = ws:scan()
			t:assertEq(#bridgeFindings(scan.findings), 0)
			local root
			for _, inst in ipairs(scan.instances) do
				if inst.componentId == "repo.host_bridge" then
					root = inst
				end
			end
			t:assert(root, "host_bridge root missing")
			-- all four methods are instances (the full bridge surface); only the two
			-- command-sending ones carry commands and sends edges
			t:assertEq(#root.ast.children, 4)
			t:assertEq(root.ast.children[1].id, "listRecipes")
			t:assertEq(root.ast.children[2].id, "applyRecipe")
			t:assertEq(root.ast.children[3].commands, {})
			t:assertEq(root.ast.methodCount, 4)
			local edges = {}
			for _, e in ipairs(scan.edges or {}) do
				edges[e.from .. "->" .. (e.to or e.symbol or "")] = e.kind
			end
			t:assertEq(edges["repo.host_bridge_method/listRecipes->repo.host_command/list"], "sends")
			t:assertEq(edges["repo.host_bridge_method/applyRecipe->repo.host_command/apply"], "sends")
		end,

		unknown_command_literal_is_unresolved_drift = function(t)
			local ws = t:workspace({
				files = baseFiles({
					[BRIDGE_TS] = bridgeTs([[
  async bogus(): Promise<void> {
    await this.send<void>({ command: 'noSuchCommand' });
  }

]]),
				}),
			})
			local scan = ws:scan()
			t:assertEq(codes(bridgeFindings(scan.findings)), {
				"relation.unresolved@repo.host_bridge_method/bogus",
			})
		end,
	},
}
