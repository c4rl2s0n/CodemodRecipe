--- Webview entry: main.ts — classify + span for overview (low mutation).

local MAIN_TS = "vscode_extension/src/webview/src/main.ts"

return {
	id = "repo.vscode_bootstrap",
	description = "VS Code webview bootstrap (main.ts createApp mount)",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles(MAIN_TS)) do
			out[#out + 1] = { path = path }
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local span = nil
		local ok = pcall(function()
			local src = c:read(path)
			if src then
				span = { from = 0, to = #src }
			end
		end)
		if not ok or not span then
			return {
				id = path,
				componentId = "repo.vscode_bootstrap",
				source = { path = path },
			}
		end
		return {
			id = path,
			componentId = "repo.vscode_bootstrap",
			source = { path = path, span = span },
		}
	end,
	relate = function(_c, _instance, index)
		local edges = { { kind = "lives_in", to = "repo.zone/vscode.webview" } }
		-- Webview mounts App.vue (honest product peer, not a fake inventory child).
		local app = "repo.vue_sfc/vscode_extension/src/webview/src/App.vue"
		if index:byId(app) then
			edges[#edges + 1] = { kind = "uses", to = app }
		end
		return edges
	end,
	toSummary = function(_c, instance)
		return {
			kind = "vscode-bootstrap",
			title = "webview main.ts",
			id = instance.ast.id,
			facts = {},
		}
	end,
}
