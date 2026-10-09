return {
	id = "repo.zone.tests",
	target = "repo.zone",
	tests = {
		identifies_zones_from_catalog = function(t)
			local ws = t:workspace({ files = {} })
			local scan = ws:scan()
			local byId = {}
			for _, inst in ipairs(scan.instances) do
				if inst.componentId == "repo.zone" then
					byId[inst.ast.id] = inst
				end
			end
			t:assert(byId["structural.modules"], "structural.modules zone missing")
			t:assertEq(byId["structural.modules"].ast.role, "structural")
			t:assert(byId["inventory.surfaces"] == nil, "inventory.surfaces removed (D-018)")
			t:assert(byId["command.adapters"], "command.adapters zone missing")
			t:assert(byId["rust.runtime"], "rust.runtime zone missing")
			t:assert(byId["vscode.extension"], "vscode.extension zone missing")
			t:assert(byId["docs.surface"], "docs.surface title-only zone missing")
			t:assertEq(byId["docs.surface"].ast.role, "documentation")
		end,
	},
}
