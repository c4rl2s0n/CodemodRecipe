local ops = require("./stack_ops")
local zonesLib = require("./lib/zones")

return {
	id = "repo-stack",
	description = "Self-hosting stack — models this repository's own structures",
	components = {
		-- zones (membership subgraph containers)
		"repo.zone",
		-- packaging + structural
		"repo.rust_crate",
		"repo.rust_mod_file",
		"repo.support_crate_surface",
		-- host domain
		"repo.host_protocol",
		"repo.protocol_key",
		"repo.mcp_tool",
		"repo.host_api_method",
		"repo.engine_edit",
		"repo.working_tree",
		"repo.recipe_registry",
		"repo.domain_type",
		"repo.host_module",
		"repo.component_pipeline",
		-- other crates
		"repo.engine_surface",
		"repo.lua_surface",
		"repo.lua_load_module",
		"repo.lua_host_api_specs",
		"repo.host_api_handlers",
		-- vscode
		"repo.host_bridge",
		"repo.vscode_command",
		"repo.vscode_manifest",
		"repo.webview_message",
		"repo.vscode_language_context",
		"repo.vscode_ext_bootstrap",
		"repo.recipe_invoke",
		"repo.ts_module",
		"repo.vue_sfc",
		"repo.vscode_bootstrap",
		-- Product tests (not Luau component *.test.luau)
		"repo.rust_test_module",
		"repo.rust_test",
		"repo.ts_test",
	},
	-- Product sources + product tests (includes crates/*/src/bin/*.rs entrypoints).
	-- Cargo.toml is identified by repo.rust_crate but excluded from the span universe.
	-- Optional `!pattern` excludes carve out non-product trees (e.g. !**/node_modules/**).
	codebase = {
		"rust/crates/*/src/**/*.rs",
		"rust/crates/*/tests/**/*.rs",
		"vscode_extension/src/**/*.ts",
		"vscode_extension/src/**/*.vue",
		"vscode_extension/package.json",
		-- Tooling / ambient (not product span)
		"!**/*.d.ts",
		"!**/*.config.ts",
	},
	zones = zonesLib.zones,
	zoneRoles = zonesLib.zoneRoles,
	relations = {
		exposes = { layout = "edge" },
		sends = { layout = "edge" },
		invokes = { layout = "edge" },
		drives = { layout = "edge" },
		stages_via = { layout = "edge" },
		feeds = { layout = "edge" },
		uses = { layout = "edge" },
		mirrors = { layout = "edge" },
		converts = { layout = "edge" },
		finalizes = { layout = "edge" },
		produces = { layout = "edge" },
		lives_in = { layout = "member" },
		handles = { layout = "edge" },
		-- Path-module / generation graph
		declares_mod = { layout = "edge" },
		imports = { layout = "edge" },
		exports = { layout = "edge" },
		tests = { layout = "edge" },
		-- Command → webview / UI surface (UI-only commands; evidence-based)
		presents = { layout = "edge" },
		-- Protocol vocabulary → host protocol root
		defines = { layout = "edge" },
	},
	operations = ops.operations,
}
