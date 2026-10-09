--- Webview Vue views and composables.

return {
	id = "repo.webview_view",
	description = "Webview Vue SFC / composable module (export leaves)",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles("vscode_extension/src/webview/**/*.vue")) do
			out[#out + 1] = { path = path }
		end
		for _, path in ipairs(c:listFiles("vscode_extension/src/webview/**/*.ts")) do
			out[#out + 1] = { path = path }
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		local ast = c:component("repo.ts_module"):parse(locator)
		if not ast then
			return nil
		end
		ast.componentId = "repo.webview_view"
		ast.id = path
		return ast
	end,
	relate = function(_c, _instance)
		return { { kind = "lives_in", to = "repo.zone/vscode.webview" } }
	end,
	toSummary = function(_c, instance)
		return {
			kind = "webview-view",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " exports" },
		}
	end,
}
