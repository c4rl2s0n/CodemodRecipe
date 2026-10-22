--- Name-form helpers for the command triangle (PascalCase ↔ camelCase ↔ snake_case).
--- Library module: no top-level id/apply/parse.

local M = {}

local COMMAND_PREFIX = "codemodRecipe."

--- Strip a leading `codemodRecipe.` prefix if present.
function M.stripCommandPrefix(s)
	if type(s) ~= "string" then
		return s
	end
	if s:sub(1, #COMMAND_PREFIX) == COMMAND_PREFIX then
		return s:sub(#COMMAND_PREFIX + 1)
	end
	return s
end

--- Split an identifier into lowercase word parts.
--- Accepts PascalCase, camelCase, snake_case, kebab-case, and mixed forms.
local function words(s)
	if type(s) ~= "string" or s == "" then
		return {}
	end
	-- normalize separators, then split on case boundaries
	local normalized = s:gsub("[-_]", " ")
	normalized = normalized:gsub("(%l)(%u)", "%1 %2")
	normalized = normalized:gsub("(%u+)(%u%l)", "%1 %2")
	local out = {}
	for w in normalized:gmatch("%S+") do
		out[#out + 1] = w:lower()
	end
	return out
end

function M.toSnake(s)
	return table.concat(words(s), "_")
end

function M.toCamel(s)
	local parts = words(s)
	if #parts == 0 then
		return ""
	end
	local out = { parts[1] }
	for i = 2, #parts do
		local w = parts[i]
		out[#out + 1] = w:sub(1, 1):upper() .. w:sub(2)
	end
	return table.concat(out)
end

function M.toPascal(s)
	local camel = M.toCamel(s)
	if camel == "" then
		return ""
	end
	return camel:sub(1, 1):upper() .. camel:sub(2)
end

--- Canonical command key used across layers (camelCase, no vscode prefix).
function M.commandKey(s)
	return M.toCamel(M.stripCommandPrefix(s))
end

return M
