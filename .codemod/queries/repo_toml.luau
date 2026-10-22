return {
	id = "repo_toml",
	queries = {
		-- [package] name = "..."
		package_name = [[
(table
  (bare_key) @table
  (pair
    (bare_key) @key
    (string) @packageName)
  (#eq? @table "package")
  (#eq? @key "name"))
]],
		-- [workspace] members = ["crates/core", ...] — one hit per member string
		workspace_members = [[
(table
  (bare_key) @table
  (pair
    (bare_key) @key
    (array
      (string) @member))
  (#eq? @table "workspace")
  (#eq? @key "members"))
]],
		-- [lints] workspace = true
		inherits_workspace_lints = [[
(table
  (bare_key) @table
  (pair
    (bare_key) @key
    (boolean) @enabled)
  (#eq? @table "lints")
  (#eq? @key "workspace"))
]],
	},
}
