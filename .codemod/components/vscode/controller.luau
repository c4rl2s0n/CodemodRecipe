--- VS Code extension controller / view modules (byte sinks first via all src files).

return {
	id = "repo.vscode_controller",
	description = "Extension TS controllers: exported symbols as coverage leaves",
	identify = function(c)
		local out = {}
		for _, path in ipairs(c:listFiles("vscode_extension/src/extension/**/*.ts")) do
			out[#out + 1] = { path = path }
		end
		return out
	end,
	parse = function(c, locator)
		local path = locator.path
		if not path then
			return nil
		end
		-- Reuse ts_module parse shape via component delegation.
		local ast = c:component("repo.ts_module"):parse(locator)
		if not ast then
			return nil
		end
		ast.componentId = "repo.vscode_controller"
		ast.id = path
		return ast
	end,
	relate = function(_c, _instance)
		return { { kind = "lives_in", to = "repo.zone/vscode.extension" } }
	end,
	toSummary = function(_c, instance)
		return {
			kind = "vscode-controller",
			title = instance.ast.id,
			id = instance.ast.id,
			facts = { #(instance.ast.children or {}) .. " exports" },
		}
	end,
}
