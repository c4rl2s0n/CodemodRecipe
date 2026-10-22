return {
	id = "remove_count_field",
	name = "remove_count_field",
	description = "Remove the count field from Settings",
	args = {
		{ name = "file", required = true, inputKind = "file" },
	},
	postExecution = {},
	apply = function(c, args)
		c:remove({
			path = args.file,
			language = "dart",
			query = [[
(class_definition
  name: (identifier) @className
  body: (class_body
    (declaration
      (initialized_identifier_list
        (initialized_identifier
          (identifier) @fieldName))) @member)
  (#eq? @className "Settings")
  (#eq? @fieldName "count"))
]],
			capture = "member",
		})
	end,
}
