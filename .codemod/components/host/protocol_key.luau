--- Wire-string constants in protocol/keys.rs (catalog + cross-layer mirror check).

local KEYS_RS = "rust/crates/host/src/protocol/keys.rs"
local TYPES_TS = "vscode_extension/src/shared/types.ts"
local MCP_INVENTORY = "rust/crates/host/src/protocol/mcp_inventory.rs"

-- Critical keys that must appear in the TS shared surface (wire property names).
local TS_MIRROR = {
	previewToken = true,
	recipe = true,
	args = true,
	ok = true,
	error = true,
	files = true,
}
-- Critical keys that should appear in MCP inventory (may be a superset of TS).
local MCP_MIRROR = {
	previewToken = true,
	recipe = true,
	inlineRecipe = true,
	args = true,
	ok = true,
	error = true,
	files = true,
}

return {
	id = "repo.protocol_key",
	description = "Host protocol wire key (pub const in keys.rs)",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles(KEYS_RS)) do
			local ok = pcall(function()
				local names = c:query({
					path = path,
					language = "rust",
					query = "repo_rust.pub_str_consts",
					capture = "constName",
				})
				local items = c:query({
					path = path,
					language = "rust",
					query = "repo_rust.pub_str_consts",
					capture = "constItem",
				})
				local values = c:query({
					path = path,
					language = "rust",
					query = "repo_rust.pub_str_consts",
					capture = "constValue",
				})
				table.sort(names, function(a, b)
					return a.span.from < b.span.from
				end)
				table.sort(items, function(a, b)
					return a.span.from < b.span.from
				end)
				table.sort(values, function(a, b)
					return a.span.from < b.span.from
				end)
				for i = 1, math.min(#names, #items, #values) do
					local raw = values[i].text or ""
					local value = raw:gsub('^"(.*)"$', "%1"):gsub("^'(.*)'$", "%1")
					out[#out + 1] = {
						path = path,
						hints = {
							span = items[i].span,
							name = names[i].text,
							value = value,
						},
					}
				end
			end)
			if not ok then
				-- skip
			end
		end
		return out
	end,
	parse = function(_c, locator)
		local path = locator.path
		local hints = locator.hints or {}
		if not path or not hints.span or not hints.name then
			return nil
		end
		return {
			id = hints.name,
			componentId = "repo.protocol_key",
			source = { path = path, span = hints.span },
			name = hints.name,
			value = hints.value or "",
		}
	end,
	verify = function(c, instance)
		local findings = {}
		local value = instance.ast.value
		local name = instance.ast.name
		-- Strip quotes from string_literal capture text if present.
		if type(value) == "string" then
			value = value:gsub('^"(.*)"$', "%1"):gsub("^'(.*)'$", "%1")
		end
		if not value or value == "" then
			return findings
		end
		local function fileHas(path, needle)
			local ok, src = pcall(function()
				return c:read(path)
			end)
			return ok and src and src:find(needle, 1, true) ~= nil
		end
		if TS_MIRROR[value] and not fileHas(TYPES_TS, value) then
			findings[#findings + 1] = {
				severity = "warning",
				code = "protocol_key.unmirrored_ts",
				message = "protocol key value '"
					.. value
					.. "' ("
					.. name
					.. ") not found in "
					.. TYPES_TS,
				instanceId = "repo.protocol_key/" .. name,
			}
		end
		if MCP_MIRROR[value] and not fileHas(MCP_INVENTORY, value) then
			findings[#findings + 1] = {
				severity = "info",
				code = "protocol_key.unmirrored_mcp",
				message = "protocol key value '"
					.. value
					.. "' ("
					.. name
					.. ") not found in MCP inventory (may be intentional)",
				instanceId = "repo.protocol_key/" .. name,
			}
		end
		return findings
	end,
	operations = {
		addKey = {
			description = "Append a pub const KEY: &str = \"value\" to protocol/keys.rs",
			args = {
				{ name = "name", required = true, help = "Const identifier (e.g. PREVIEW_TOKEN)" },
				{ name = "value", required = true, help = "Wire string (e.g. previewToken)" },
				{ name = "ifExists", required = false, help = "skip (default) | error" },
			},
			apply = function(c, args, _target, graph)
				local name = args.name
				local value = args.value
				if not name or name == "" or not value or value == "" then
					error("addKey requires args.name and args.value")
				end
				for _, existing in ipairs(graph:byComponent("repo.protocol_key")) do
					if existing.ast.name == name or existing.ast.value == value then
						if args.ifExists == "error" then
							error("protocol key already exists: " .. name)
						end
						return
					end
				end
				c:insert({
					path = KEYS_RS,
					language = "rust",
					query = "(source_file) @file",
					capture = "file",
					anchor = "end",
					text = "\npub const " .. name .. ': &str = "' .. value .. '";\n',
				})
			end,
		},
	},
	relate = function(_c, _instance, index)
		local edges = { { kind = "lives_in", to = "repo.zone/command.adapters" } }
		-- Keys are the wire vocabulary SSOT for host protocol.
		local to = "repo.host_protocol/host-protocol"
		if not index or index:byId(to) then
			edges[#edges + 1] = { kind = "defines", to = to }
		end
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "protocol-key",
			title = instance.ast.name,
			id = instance.ast.id,
			facts = { instance.ast.value or "" },
		}
	end,
}
