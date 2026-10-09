--- Webview ↔ extension message protocol (consts, unions, handlers, client posts).

local MESSAGES = "vscode_extension/src/shared/messages.ts"
local HANDLERS = "vscode_extension/src/extension/views/recipeRunnerHandlers.ts"
local CLIENT = "vscode_extension/src/webview/src/extensionClient.ts"

local function constKeys(c, path, constName)
	local keys = {}
	local src = c:read(path)
	-- Rough: KEY: 'value' inside WEBVIEW_TO_EXTENSION / EXTENSION_TO_WEBVIEW
	local block = src:match(constName .. "%s*=%s*{(.-)}%s*as%s*const")
	if not block then
		return keys
	end
	for key in block:gmatch("([%w_]+)%s*:") do
		keys[#keys + 1] = key
	end
	table.sort(keys)
	return keys
end

local function handlerCases(c)
	local cases = {}
	local ok, src = pcall(function()
		return c:read(HANDLERS)
	end)
	if not ok or not src then
		return cases
	end
	for typ in src:gmatch("case%s+WEBVIEW_TO_EXTENSION%.([%w_]+)") do
		cases[typ] = true
	end
	return cases
end

local function clientPosts(c)
	local posts = {}
	local ok, src = pcall(function()
		return c:read(CLIENT)
	end)
	if not ok or not src then
		return posts
	end
	for typ in src:gmatch("WEBVIEW_TO_EXTENSION%.([%w_]+)") do
		posts[typ] = true
	end
	return posts
end

return {
	id = "repo.webview_message",
	description = "Webview message protocol: consts ↔ unions ↔ handlers ↔ client",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles(MESSAGES)) do
			out[#out + 1] = { path = path }
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local toExt = constKeys(c, path, "WEBVIEW_TO_EXTENSION")
		local toWeb = constKeys(c, path, "EXTENSION_TO_WEBVIEW")
		local identity = {
			componentId = "repo.webview_message",
			id = "webview-messages",
			path = path,
		}
		local children = {}
		local keySpans = {}
		local okKeys = pcall(function()
			local names = c:query({
				path = path,
				language = "typescript",
				query = "repo_typescript.as_const_object_keys",
				capture = "key",
			})
			for _, hit in ipairs(names) do
				keySpans[hit.text] = hit.span
			end
		end)
		for _, key in ipairs(toExt) do
			local source = { path = path }
			if okKeys and keySpans[key] then
				source.span = keySpans[key]
			end
			local child = c:component("repo.webview_message_entry"):parse({
				source = source,
				parent = identity,
				role = "message",
				hints = {
					id = "toExt." .. key,
					direction = "toExtension",
					name = key,
				},
			})
			if child then
				child.componentId = "repo.webview_message_entry"
				child.role = "message"
				children[#children + 1] = child
			end
		end
		for _, key in ipairs(toWeb) do
			local source = { path = path }
			if okKeys and keySpans[key] then
				source.span = keySpans[key]
			end
			local child = c:component("repo.webview_message_entry"):parse({
				source = source,
				parent = identity,
				role = "message",
				hints = {
					id = "toWeb." .. key,
					direction = "toWebview",
					name = key,
				},
			})
			if child then
				child.componentId = "repo.webview_message_entry"
				child.role = "message"
				children[#children + 1] = child
			end
		end
		return {
			id = "webview-messages",
			componentId = "repo.webview_message",
			source = { path = path },
			toExtension = toExt,
			toWebview = toWeb,
			children = children,
		}
	end,
	verify = function(c, instance)
		local findings = {}
		local cases = handlerCases(c)
		local posts = clientPosts(c)
		for _, key in ipairs(instance.ast.toExtension or {}) do
			if not cases[key] then
				findings[#findings + 1] = {
					severity = "warning",
					code = "webview_message.missing_handler",
					message = "WEBVIEW_TO_EXTENSION." .. key .. " has no handler case",
					instanceId = instance.id,
				}
			end
			if not posts[key] then
				findings[#findings + 1] = {
					severity = "info",
					code = "webview_message.unused_client",
					message = "WEBVIEW_TO_EXTENSION."
						.. key
						.. " not referenced from extensionClient (may be fine)",
					instanceId = instance.id,
				}
			end
		end
		return findings
	end,
	toSummary = function(_c, instance)
		return {
			kind = "webview-messages",
			title = "Webview messages",
			id = instance.ast.id,
			facts = {
				#(instance.ast.toExtension or {}) .. " →ext",
				#(instance.ast.toWebview or {}) .. " →web",
			},
		}
	end,
}
