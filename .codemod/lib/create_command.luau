--- Layer-local createCommand helpers (host / mcp / vscode).
--- Library module: no top-level id/apply/parse.

local names = require("./names")

local MOD_RS = "rust/crates/host/src/protocol/mod.rs"
local DISPATCH_RS = "rust/crates/host/src/protocol/dispatch/mod.rs"
local TYPES_TS = "vscode_extension/src/shared/types.ts"
local MCP_RS = "rust/crates/host/src/bin/codemod_mcp.rs"
local INVENTORY_RS = "rust/crates/host/src/protocol/mcp_inventory.rs"
local PACKAGE_JSON = "vscode_extension/package.json"
local CONSTANTS_TS = "vscode_extension/src/extension/constants.ts"
local EXTENSION_TS = "vscode_extension/src/extension/extension.ts"

local M = {}

local function commandForms(rawName)
	local key = names.commandKey(rawName)
	return {
		key = key,
		variant = names.toPascal(key),
		rename = key,
		snake = names.toSnake(key),
		constKey = key,
		commandId = "codemodRecipe." .. key,
		mcpTool = names.toSnake(key),
		title = names.toPascal(key),
	}
end

function M.forms(rawName)
	return commandForms(rawName)
end

function M.hostHasCommand(graph, rename)
	for _, hc in ipairs(graph:byComponent("repo.host_command")) do
		if hc.ast.rename == rename or hc.ast.variant == rename then
			return true
		end
	end
	return false
end

function M.mcpHasTool(graph, toolName)
	for _, entry in ipairs(graph:byComponent("repo.mcp_tool_entry")) do
		if entry.ast.id == toolName then
			return true
		end
	end
	return false
end

function M.vscodeHasCommand(graph, commandId)
	for _, entry in ipairs(graph:byComponent("repo.vscode_command_entry")) do
		if entry.ast.commandId == commandId or entry.ast.id == commandId then
			return true
		end
	end
	return false
end

local function skipOrError(exists, label, ifExists)
	if not exists then
		return false
	end
	if ifExists == "error" then
		error(label .. " already exists")
	end
	return true
end

local function writeFile(c, path, content)
	-- Whole-file replace: delete+create is rejected when the path was already
	-- touched earlier in the same WorkingTree session (multi-layer fan-out).
	local lang, root = "rust", "source_file"
	if path:match("%.ts$") then
		lang, root = "typescript", "program"
	elseif path:match("%.json$") then
		lang, root = "json", "document"
	end
	c:replace({
		path = path,
		language = lang,
		query = "(" .. root .. ") @file",
		capture = "file",
		text = content,
	})
end

--- Host layer: enum variant + dispatch stub + TS union member.
function M.applyHost(c, args, graph)
	local forms = commandForms(args.name)
	if skipOrError(
		M.hostHasCommand(graph, forms.rename),
		"host command '" .. forms.rename .. "'",
		args.ifExists
	) then
		return
	end

	c:insert({
		path = MOD_RS,
		language = "rust",
		query = "repo_rust.host_command_enum_body",
		capture = "body",
		anchor = "end",
		text = "\n    #[serde(rename = \""
			.. forms.rename
			.. "\")]\n    "
			.. forms.variant
			.. " {\n        #[serde(default)]\n        path: String,\n    },",
	})

	c:insert({
		path = DISPATCH_RS,
		language = "rust",
		query = "repo_rust.host_command_dispatch_match",
		capture = "match",
		anchor = "end",
		text = "\n        HostCommand::"
			.. forms.variant
			.. " { path } => {\n            let _ = path;\n            serde_json::json!({\n                \"ok\": false,\n                \"error\": \"HostCommand::"
			.. forms.variant
			.. " not implemented\"\n            })\n        }",
	})

	local decls = c:query({
		path = TYPES_TS,
		language = "typescript",
		query = "repo_typescript.host_command_alias_decl",
		capture = "aliasDecl",
	})
	local decl = decls[1]
	if not decl then
		error("HostCommand type alias not found in " .. TYPES_TS)
	end
	local members = {}
	for _, hit in ipairs(c:query({
		path = TYPES_TS,
		language = "typescript",
		query = "repo_typescript.host_command_union_members",
		capture = "tsCommand",
	})) do
		if hit.span.from >= decl.span.from and hit.span.to <= decl.span.to then
			members[#members + 1] = hit
		end
	end
	table.sort(members, function(a, b)
		return a.span.from < b.span.from
	end)
	if #members == 0 then
		error("no HostCommand union members in " .. TYPES_TS)
	end
	local lastCmd = members[#members].text
	-- Replace the trailing `;` of the alias with ` | { new };` via whole-alias rewrite.
	local source = c:read(TYPES_TS)
	local aliasText = source:sub(decl.span.from + 1, decl.span.to)
	local trimmed = aliasText:gsub("%s*;%s*$", "")
	local nextAlias = trimmed
		.. "\n  | { command: '"
		.. forms.rename
		.. "'; path?: string };"
	local next = source:sub(1, decl.span.from) .. nextAlias .. source:sub(decl.span.to + 1)
	writeFile(c, TYPES_TS, next)
end

--- MCP layer: inventory entry + tools/call arm.
function M.applyMcp(c, args, graph)
	local forms = commandForms(args.name)
	if skipOrError(
		M.mcpHasTool(graph, forms.mcpTool),
		"mcp tool '" .. forms.mcpTool .. "'",
		args.ifExists
	) then
		return
	end

	local inv = c:read(INVENTORY_RS)
	local metaEntry = [[
        McpToolMeta {
            name: "]] .. forms.mcpTool .. [[",
            description: "Auto-scaffolded tool for HostCommand::]] .. forms.variant .. [[.",
            host_command: Some("]] .. forms.variant .. [["),
        },
]]
	local metaAnchor = 'name: "run_tests"'
	local metaPos = inv:find(metaAnchor, 1, true)
	if not metaPos then
		error("inventory anchor run_tests not found in " .. INVENTORY_RS)
	end
	local objStart = metaPos
	while objStart > 1 and inv:sub(objStart, objStart + #"McpToolMeta" - 1) ~= "McpToolMeta" do
		objStart -= 1
	end
	inv = inv:sub(1, objStart - 1) .. metaEntry .. inv:sub(objStart)

	local schemaArm = '        "'
		.. forms.mcpTool
		.. '" => json!({ "type": "object", "properties": { "path": { "type": "string" } } }),\n'
	local schemaAnchor = '        "run_tests" =>'
	local schemaPos = inv:find(schemaAnchor, 1, true)
	if not schemaPos then
		error("input_schema anchor run_tests not found in " .. INVENTORY_RS)
	end
	inv = inv:sub(1, schemaPos - 1) .. schemaArm .. inv:sub(schemaPos)
	writeFile(c, INVENTORY_RS, inv)

	local src = c:read(MCP_RS)
	local callArm = '                "'
		.. forms.mcpTool
		.. '" => handle_command(registry, HostCommand::'
		.. forms.variant
		.. ' {\n                    path: arguments.get("path").and_then(|v| v.as_str()).filter(|s| !s.is_empty()).map(String::from),\n                }),\n'
	local callAnchor = '                "run_tests" =>'
	local callPos = src:find(callAnchor, 1, true)
	if not callPos then
		error("tools/call anchor run_tests not found in " .. MCP_RS)
	end
	src = src:sub(1, callPos - 1) .. callArm .. src:sub(callPos)
	writeFile(c, MCP_RS, src)
end

--- VS Code layer: package.json + COMMANDS const + registerCommand stub.
function M.applyVscode(c, args, graph)
	local forms = commandForms(args.name)
	if skipOrError(
		M.vscodeHasCommand(graph, forms.commandId),
		"vscode command '" .. forms.commandId .. "'",
		args.ifExists
	) then
		return
	end

	local pkg = c:read(PACKAGE_JSON)
	local pkgEntry = [[
      {
        "command": "]] .. forms.commandId .. [[",
        "title": "Codemod: ]] .. forms.title .. [["
      },]]
	local _, afterBracket = pkg:find('"commands"%s*:%s*%[')
	if not afterBracket then
		error("contributes.commands array not found in " .. PACKAGE_JSON)
	end
	pkg = pkg:sub(1, afterBracket) .. "\n" .. pkgEntry .. pkg:sub(afterBracket + 1)
	writeFile(c, PACKAGE_JSON, pkg)

	c:insert({
		path = CONSTANTS_TS,
		language = "typescript",
		query = "repo_typescript.commands_const_object",
		capture = "commandsObj",
		anchor = "end",
		text = "\n  " .. forms.constKey .. ": '" .. forms.commandId .. "',",
	})

	local ext = c:read(EXTENSION_TS)
	local stub = "\n    vscode.commands.registerCommand(COMMANDS."
		.. forms.constKey
		.. ", () => {\n      /* TODO: wire "
		.. forms.commandId
		.. " */\n    }),"
	local pos, lastEnd = 1, nil
	while true do
		local s, e = ext:find("vscode%.commands%.registerCommand%(COMMANDS%.[%w_]+,", pos)
		if not s then
			break
		end
		local close = ext:find("%),", e) or ext:find("%);", e)
		if not close then
			break
		end
		lastEnd = close + 1
		pos = close + 2
	end
	if not lastEnd then
		error("no registerCommand(COMMANDS.*) site found in " .. EXTENSION_TS)
	end
	ext = ext:sub(1, lastEnd) .. stub .. ext:sub(lastEnd + 1)
	writeFile(c, EXTENSION_TS, ext)
end

return M
