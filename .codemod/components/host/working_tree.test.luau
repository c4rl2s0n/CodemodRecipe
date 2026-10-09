return {
	id = "repo.working_tree.tests",
	target = "repo.working_tree",
	tests = {
		identifies_working_tree = function(t)
			local ws = t:workspace({
				files = {
					["rust/crates/host/src/engine/working_tree.rs"] = [[
pub struct WorkingTree {}
impl WorkingTree {
    pub fn finalize(&self) {}
}
]],
				},
			})
			local scan = ws:scan()
			local found
			for _, inst in ipairs(scan.instances) do
				if inst.componentId == "repo.working_tree" then
					found = inst
				end
			end
			t:assert(found, "working_tree missing")
			t:assert(found.ast.hasWorkingTree)
			t:assert(found.ast.hasFinalize)
		end,
	},
}
