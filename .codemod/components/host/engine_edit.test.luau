return {
	id = "repo.engine_edit.tests",
	target = "repo.engine_edit",
	tests = {
		identifies_engine = function(t)
			local ws = t:workspace({
				files = {
					["rust/crates/engine/src/engine/mod.rs"] = [[
pub struct Engine {}
]],
					["rust/crates/engine/src/ops.rs"] = [[
pub enum EditOp { Insert }
]],
				},
			})
			local scan = ws:scan()
			local found
			for _, inst in ipairs(scan.instances) do
				if inst.componentId == "repo.engine_edit" then
					found = inst
				end
			end
			t:assert(found, "engine_edit missing")
			t:assert(found.ast.hasEngine)
			t:assert(found.ast.hasEditOp)
		end,
	},
}
