return {
	id = "repo_json",
	queries = {
		-- vscode_extension/package.json contributes.commands[].command ids.
		manifest_commands = [[
(pair
  key: (string
    (string_content) @section)
  value: (object
    (pair
      key: (string
        (string_content) @group)
      value: (array
        (object
          (pair
            key: (string
              (string_content) @key)
            value: (string
              (string_content) @commandId))))))
  (#eq? @section "contributes")
  (#eq? @group "commands")
  (#eq? @key "command"))
]],
		-- The contributes.commands array itself (for createCommand insert-at-end).
		manifest_commands_array = [[
(pair
  key: (string
    (string_content) @section)
  value: (object
    (pair
      key: (string
        (string_content) @group)
      value: (array) @commandsArray))
  (#eq? @section "contributes")
  (#eq? @group "commands"))
]],
		-- Top-level document pairs (key + whole pair span).
		-- value: (_) required so @pair can sit on the pair node (capture-after-fields).
		top_level_pairs = [[
(document
  (object
    (pair
      key: (string
        (string_content) @key)
      value: (_)) @pair))
]],
		-- contributes.<group> pairs (group name + whole pair under contributes).
		contributes_groups = [[
(pair
  key: (string
    (string_content) @section)
  value: (object
    (pair
      key: (string
        (string_content) @group)
      value: (_)) @pair)
  (#eq? @section "contributes"))
]],
	},
}
