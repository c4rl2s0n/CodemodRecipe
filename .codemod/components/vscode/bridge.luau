local BRIDGE_TS = "vscode_extension/src/extension/host/hostBridge.ts"

local function contains(outer, inner)
	return inner.from >= outer.from and inner.to <= outer.to
end

--- method name -> whole-method span, paired by document order
local function collectMethods(c, path)
	local names = c:query({
		path = path,
		language = "typescript",
		query = "repo_typescript.bridge_methods",
		capture = "methodName",
	})
	local spans = c:query({
		path = path,
		language = "typescript",
		query = "repo_typescript.bridge_methods",
		capture = "method",
	})
	table.sort(names, function(a, b)
		return a.span.from < b.span.from
	end)
	table.sort(spans, function(a, b)
		return a.span.from < b.span.from
	end)
	local out = {}
	for i = 1, math.min(#names, #spans) do
		out[#out + 1] = { name = names[i].text, span = spans[i].span }
	end
	return out
end

return {
	id = "repo.host_bridge",
	description = "VS Code host bridge: methods sending HostCommand payloads to the Rust host",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles(BRIDGE_TS)) do
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
			componentId = "repo.host_bridge",
			id = "host-bridge",
			path = path,
		}

		-- pair @sendFn with @commandLiteral by document order; only real senders count
		local SEND_FNS = { send = true, sendPersistent = true, sendOneShot = true }
		local sendFns = c:query({
			path = path,
			language = "typescript",
			query = "repo_typescript.bridge_command_literals",
			capture = "sendFn",
		})
		local literalHits = c:query({
			path = path,
			language = "typescript",
			query = "repo_typescript.bridge_command_literals",
			capture = "commandLiteral",
		})
		table.sort(sendFns, function(a, b)
			return a.span.from < b.span.from
		end)
		table.sort(literalHits, function(a, b)
			return a.span.from < b.span.from
		end)
		local literals = {}
		for i = 1, math.min(#sendFns, #literalHits) do
			if SEND_FNS[sendFns[i].text] then
				literals[#literals + 1] = { command = literalHits[i].text, span = literalHits[i].span }
			end
		end

		-- every method becomes an instance (the bridge surface), not only
		-- command-sending ones — invokes edges from vscode_command_entry target
		-- non-sending methods too (ensureHost etc.)
		local children = {}
		local methodCount = 0
		for _, method in ipairs(collectMethods(c, path)) do
			methodCount += 1
			local commands, seen = {}, {}
			for _, lit in ipairs(literals) do
				if contains(method.span, lit.span) and not seen[lit.command] then
					seen[lit.command] = true
					commands[#commands + 1] = lit.command
				end
			end
			local child = c:component("repo.host_bridge_method"):parse({
				source = { path = path, span = method.span },
				parent = identity,
				role = "method",
				hints = { method = method.name, commands = commands },
			})
			if child then
				child.componentId = "repo.host_bridge_method"
				child.role = "method"
				children[#children + 1] = child
			end
		end

		return {
			id = "host-bridge",
			componentId = "repo.host_bridge",
			source = { path = path },
			children = children,
			methodCount = methodCount,
		}
	end,
	relate = function(_c, _instance)
		return { { kind = "lives_in", to = "repo.zone/command.adapters" } }
	end,
	toSummary = function(_c, instance)
		local ast = instance.ast
		return {
			kind = "host-bridge",
			title = "hostBridge.ts",
			id = ast.id,
			path = ast.source and ast.source.path or nil,
			facts = {
				#(ast.children or {}) .. " command-sending methods",
				(ast.methodCount or 0) .. " methods total",
			},
		}
	end,
}
