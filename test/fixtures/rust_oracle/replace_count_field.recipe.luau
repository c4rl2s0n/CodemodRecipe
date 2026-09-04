return {
	id = "replace_count_field",
	name = "replace_count_field",
	description = "Replace the count field initializer (idempotent when unchanged)",
	args = {
		{ name = "file", required = true, inputKind = "file" },
	},
	postExecution = {},
	apply = function(c, args)
		c:replace({
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
			text = "  final int count = 0;",
		})
	end,
}
