return {
	id = "repo.rust_crate.tests",
	target = "repo.rust_crate",
	tests = {
		parses_package_name_and_lints = function(t)
			local ws = t:workspace({
				files = {
					["rust/Cargo.toml"] = '[workspace]\nmembers = ["crates/core"]\n',
					["rust/crates/core/Cargo.toml"] = '[package]\nname = "codemod_recipe_core"\n\n[lints]\nworkspace = true\n',
				},
			})
			local node = ws:parse(
				"repo.rust_crate",
				{ path = "rust/crates/core/Cargo.toml", crate = "core" }
			)
			t:assertEq(node.dir, "core")
			t:assertEq(node.packageName, "codemod_recipe_core")
			t:assertEq(node.inheritsWorkspaceLints, true)
		end,

		verify_clean_when_listed_and_conventional = function(t)
			local ws = t:workspace({
				files = {
					["rust/Cargo.toml"] = '[workspace]\nmembers = ["crates/core"]\n',
					["rust/crates/core/Cargo.toml"] = '[package]\nname = "codemod_recipe_core"\n\n[lints]\nworkspace = true\n',
				},
			})
			local scan = ws:scan()
			local crates = 0
			for _, inst in ipairs(scan.instances) do
				if inst.componentId == "repo.rust_crate" then
					crates = crates + 1
				end
			end
			t:assertEq(crates, 1)
			t:assertEq(#scan.findings, 0)
		end,

		verify_flags_unlisted_unconventional_crate = function(t)
			local ws = t:workspace({
				files = {
					["rust/Cargo.toml"] = "[workspace]\nmembers = []\n",
					["rust/crates/rogue/Cargo.toml"] = '[package]\nname = "rogue"\n',
				},
			})
			local scan = ws:scan()
			local crates = 0
			for _, inst in ipairs(scan.instances) do
				if inst.componentId == "repo.rust_crate" then
					crates = crates + 1
				end
			end
			t:assertEq(crates, 1)
			local codes = {}
			for _, f in ipairs(scan.findings) do
				codes[f.code] = true
			end
			t:assert(codes["crate.not_in_workspace_members"], "expected not_in_workspace_members finding")
			t:assert(codes["crate.package_name_convention"], "expected package_name_convention finding")
			t:assert(codes["crate.missing_workspace_lints"], "expected missing_workspace_lints finding")
		end,

		verify_flags_forbidden_path_dep = function(t)
			local ws = t:workspace({
				files = {
					["rust/Cargo.toml"] = '[workspace]\nmembers = ["crates/lua"]\n',
					["rust/crates/lua/Cargo.toml"] = [[
[package]
name = "codemod_recipe_lua"

[dependencies]
codemod_recipe_engine = { path = "../engine" }

[lints]
workspace = true
]],
				},
			})
			local scan = ws:scan()
			local codes = {}
			for _, f in ipairs(scan.findings) do
				codes[f.code] = true
			end
			t:assert(codes["crate.forbidden_path_dep"], "expected forbidden_path_dep (lua→engine)")
		end,
	},
}
