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
		"repo.host_api_handlers",
		-- vscode
		"repo.host_bridge",
		"repo.vscode_command",
		"repo.webview_message",
		"repo.ts_module",
		-- specialized vscode roots (compose on ts_module parse; listed for domain verify later)
		-- avoid triple-identify: ts_module already covers extension + webview + shared
	},
	-- Production sources only (exclude crate integration tests)
	codebase = {
		"rust/crates/*/src/**/*.rs",
		"rust/crates/*/Cargo.toml",
		"vscode_extension/src/**/*.ts",
		"vscode_extension/src/**/*.vue",
		"vscode_extension/package.json",
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
	},
	operations = ops.operations,
}
