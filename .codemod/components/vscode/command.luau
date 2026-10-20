local PACKAGE_JSON = "vscode_extension/package.json"
local CONSTANTS_TS = "vscode_extension/src/extension/constants.ts"
local SRC_GLOB = "vscode_extension/src/**/*.ts"
local createCommand = require("../../lib/create_command")

--- commandId -> constKey (from COMMANDS in constants.ts)
local function collectConstants(c)
	local byId = {}
	for _, file in ipairs(c:listFiles(CONSTANTS_TS)) do
		local keys = c:query({
			path = file,
			language = "typescript",
			query = "repo_typescript.commands_constants",
			capture = "constKey",
		})
		local ids = c:query({
			path = file,
			language = "typescript",
			query = "repo_typescript.commands_constants",
			capture = "commandId",
		})
		-- pair by span: each pair contributes one key and one id, in document order
		table.sort(keys, function(a, b)
			return a.span.from < b.span.from
		end)
		table.sort(ids, function(a, b)
			return a.span.from < b.span.from
		end)
		for i = 1, math.min(#keys, #ids) do
			byId[ids[i].text] = keys[i].text
		end
	end
	return byId
end

local function contains(outer, inner)
	return inner.from >= outer.from and inner.to <= outer.to
end

--- Classify a call callee by its text: bridge method / this-local / bare local / external.
--- (Awaited generic calls nest `await` in the function field — strip the prefix.)
local function classifyCallee(text)
	local t = text:gsub("^await%s+", "")
	local m = t:match("^bridge%.([%w_]+)$")
		or t:match("^this%.bridge%.([%w_]+)$")
		or t:match("^hostBridge%.([%w_]+)$")
		or t:match("^this%.hostBridge%.([%w_]+)$")
		-- Common aliases: `const b = bridge` / runner helpers still use `.method`.
		or t:match("%.bridge%.([%w_]+)$")
	if m then
		return "bridge", m
	end
	m = t:match("^this%.([%w_]+)$")
	if m then
		return "this", m
	end
	-- Bare local: bootstrap(), refreshRecipes(), runRecipeFromExplorer(...)
	m = t:match("^([%w_]+)$")
	if m then
		return "local", m
	end
	return "external", nil
end

--- All bridge methods reachable from a span, following this.* methods and same-file
--- local function declarations (cycle-safe). complete=false when a call stays unresolved.
local function resolveSpanCalls(span, allCalls, methodsByName, localsByName)
	local bridge, seenBridge = {}, {}
	local complete, anyCalls = true, false
	local visited = {}
	local queue = { span }
	while #queue > 0 do
		local cur = table.remove(queue)
		for _, call in ipairs(allCalls) do
			if contains(cur, call.span) then
				anyCalls = true
				local kind, name = classifyCallee(call.text)
				-- Lifecycle helpers (dispose) are not HostCommand send wrappers.
				if kind == "bridge" and name ~= "dispose" then
					if not seenBridge[name] then
						seenBridge[name] = true
						bridge[#bridge + 1] = name
					end
				elseif kind == "this" or kind == "local" then
					local mspan = methodsByName[name] or (localsByName and localsByName[name])
					local visitKey = kind .. ":" .. name
					if mspan and not visited[visitKey] then
						visited[visitKey] = true
						queue[#queue + 1] = mspan
					elseif not mspan then
						complete = false
					end
				else
					complete = false
				end
			end
		end
	end
	table.sort(bridge)
	return bridge, complete, anyCalls
end

--- constKey -> { file, bridgeCalls, analyzed, complete, empty }
local function collectRegistrationDetails(c)
	local details = {}
	for _, file in ipairs(c:listFiles(SRC_GLOB)) do
		local keys = c:query({
			path = file,
			language = "typescript",
			query = "repo_typescript.command_registration_handlers",
			capture = "registeredKey",
		})
		local handlers = c:query({
			path = file,
			language = "typescript",
			query = "repo_typescript.command_registration_handlers",
			capture = "handler",
		})
		if #keys > 0 then
			table.sort(keys, function(a, b)
				return a.span.from < b.span.from
			end)
			table.sort(handlers, function(a, b)
				return a.span.from < b.span.from
			end)

			local allCalls = c:query({
				path = file,
				language = "typescript",
				query = "repo_typescript.all_calls",
				capture = "callee",
			})
			local methodNames = c:query({
				path = file,
				language = "typescript",
				query = "repo_typescript.bridge_methods",
				capture = "methodName",
			})
			local methodSpans = c:query({
				path = file,
				language = "typescript",
				query = "repo_typescript.bridge_methods",
				capture = "method",
			})
			table.sort(methodNames, function(a, b)
				return a.span.from < b.span.from
			end)
			table.sort(methodSpans, function(a, b)
				return a.span.from < b.span.from
			end)
			local methodsByName = {}
			for i = 1, math.min(#methodNames, #methodSpans) do
				methodsByName[methodNames[i].text] = methodSpans[i].span
			end

			-- Module-level function declarations / const arrows for bare-call following.
			local localsByName = {}
			local function ingestLocals(queryId)
				local fnNames = c:query({
					path = file,
					language = "typescript",
					query = queryId,
					capture = "fnName",
				})
				local fns = c:query({
					path = file,
					language = "typescript",
					query = queryId,
					capture = "fn",
				})
				table.sort(fnNames, function(a, b)
					return a.span.from < b.span.from
				end)
				table.sort(fns, function(a, b)
					return a.span.from < b.span.from
				end)
				for i = 1, math.min(#fnNames, #fns) do
					localsByName[fnNames[i].text] = fns[i].span
				end
			end
			pcall(function()
				ingestLocals("repo_typescript.function_declarations")
				ingestLocals("repo_typescript.const_function_bindings")
			end)

			-- Detect webview / UI presentation hints in the file (for presents edges).
			local srcOk, src = pcall(function()
				return c:read(file)
			end)
			local touchesWebview = srcOk
				and src
				and (
					src:find("webview", 1, true)
					or src:find("postMessage", 1, true)
					or src:find("Webview", 1, true)
				)

			for i = 1, math.min(#keys, #handlers) do
				local handler = handlers[i]
				local detail = {
					file = file,
					bridgeCalls = {},
					analyzed = false,
					complete = false,
					empty = false,
					touchesWebview = touchesWebview or false,
				}
				if handler.kind == "arrow_function" or handler.kind == "function_expression" then
					local bridgeCalls, complete, anyCalls =
						resolveSpanCalls(handler.span, allCalls, methodsByName, localsByName)
					detail.analyzed = true
					detail.complete = complete
					detail.empty = not anyCalls
					detail.bridgeCalls = bridgeCalls
				end
				details[keys[i].text] = detail
			end
		end
	end
	return details
end

--- literal command id -> registering file (bypasses the COMMANDS const)
local function collectLiteralRegistrations(c)
	local literals = {}
	for _, file in ipairs(c:listFiles(SRC_GLOB)) do
		for _, hit in ipairs(c:query({
			path = file,
			language = "typescript",
			query = "repo_typescript.command_registrations_literal",
			capture = "literalId",
		})) do
			literals[hit.text] = file
		end
	end
	return literals
end

return {
	id = "repo.vscode_command",
	description = "VS Code command wiring: package.json manifest ↔ COMMANDS const ↔ registerCommand sites ↔ host bridge calls",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles(PACKAGE_JSON)) do
			out[#out + 1] = { path = path }
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local identity = {
			componentId = "repo.vscode_command",
			id = "vscode-commands",
			path = path,
		}

		local constantsById = collectConstants(c)
		local registrationDetails = collectRegistrationDetails(c)

		local manifest = c:query({
			path = path,
			language = "json",
			query = "repo_json.manifest_commands",
			capture = "commandId",
		})

		local children = {}
		for _, hit in ipairs(manifest) do
			local constKey = constantsById[hit.text]
			local detail = constKey and registrationDetails[constKey] or nil
			local child = c:component("repo.vscode_command_entry"):parse({
				source = { path = path, span = hit.span },
				parent = identity,
				role = "command",
				hints = {
					commandId = hit.text,
					bridgeCalls = detail and detail.bridgeCalls or {},
					handlerAnalyzed = detail and detail.analyzed or false,
					handlerEmpty = detail and detail.empty or false,
					touchesWebview = detail and detail.touchesWebview or false,
					registrationFile = detail and detail.file or nil,
				},
			})
			if child then
				child.componentId = "repo.vscode_command_entry"
				child.role = "command"
				children[#children + 1] = child
			end
		end

		-- flat constKey -> file view for the wiring checks
		local registrations = {}
		for constKey, detail in pairs(registrationDetails) do
			registrations[constKey] = detail.file
		end

		return {
			id = "vscode-commands",
			componentId = "repo.vscode_command",
			source = { path = path },
			children = children,
			constantsById = constantsById,
			registrations = registrations,
			literalRegistrations = collectLiteralRegistrations(c),
		}
	end,
	verify = function(_c, instance)
		local findings = {}
		local ast = instance.ast
		local constantsById = ast.constantsById or {}
		local registrations = ast.registrations or {}
		local manifestIds = {}

		for _, child in ipairs(ast.children or {}) do
			if child.componentId == "repo.vscode_command_entry" then
				local commandId = child.commandId
				manifestIds[commandId] = true
				local childId = "repo.vscode_command_entry/" .. commandId
				local constKey = constantsById[commandId]
				if not constKey then
					findings[#findings + 1] = {
						severity = "error",
						code = "vscode_command.missing_constant",
						message = "manifest command '" .. commandId .. "' has no COMMANDS entry in constants.ts",
						instanceId = childId,
					}
				elseif not registrations[constKey] then
					findings[#findings + 1] = {
						severity = "error",
						code = "vscode_command.missing_registration",
						message = "manifest command '"
							.. commandId
							.. "' (COMMANDS."
							.. constKey
							.. ") is never registered",
						instanceId = childId,
					}
				elseif child.handlerAnalyzed and child.handlerEmpty then
					findings[#findings + 1] = {
						severity = "warning",
						code = "vscode_command.empty_handler",
						message = "manifest command '" .. commandId .. "' is registered with an empty handler body",
						instanceId = childId,
					}
				end
			end
		end

		for literalId, file in pairs(ast.literalRegistrations or {}) do
			findings[#findings + 1] = {
				severity = "warning",
				code = "vscode_command.literal_registration",
				message = "'"
					.. literalId
					.. "' is registered by string literal in "
					.. file
					.. " — add a COMMANDS entry and register via the const",
				instanceId = instance.id,
			}
		end

		for commandId, constKey in pairs(constantsById) do
			local childId = "repo.vscode_command_entry/" .. commandId
			if not manifestIds[commandId] then
				findings[#findings + 1] = {
					severity = "warning",
					code = "vscode_command.hidden_command",
					message = "COMMANDS."
						.. constKey
						.. " ('"
						.. commandId
						.. "') is not in package.json contributes.commands (internal-only command?)",
					instanceId = childId,
				}
			end
			if not registrations[constKey] then
				findings[#findings + 1] = {
					severity = "warning",
					code = "vscode_command.dead_constant",
					message = "COMMANDS." .. constKey .. " is never passed to registerCommand",
					instanceId = childId,
				}
			end
		end

		return findings
	end,
	operations = {
		createCommand = {
			description = "Add package.json command + COMMANDS entry + registerCommand stub",
			args = {
				{ name = "name", required = true },
				{ name = "ifExists", required = false },
			},
			apply = function(c, args, _target, graph)
				createCommand.applyVscode(c, args, graph)
			end,
		},
	},
	relate = function(_c, _instance)
		return { { kind = "lives_in", to = "repo.zone/command.adapters" } }
	end,
	toSummary = function(_c, instance)
		local ast = instance.ast
		local manifest = 0
		for _, child in ipairs(ast.children or {}) do
			if child.componentId == "repo.vscode_command_entry" then
				manifest += 1
			end
		end
		local registered = 0
		for _ in pairs(ast.registrations or {}) do
			registered += 1
		end
		return {
			kind = "vscode-commands",
			title = "VS Code commands",
			id = ast.id,
			path = ast.source and ast.source.path or nil,
			facts = {
				manifest .. " manifest entries",
				registered .. " registrations",
			},
		}
	end,
}
