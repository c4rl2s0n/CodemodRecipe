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
	},
}
