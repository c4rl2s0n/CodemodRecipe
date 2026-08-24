return {
	id = "insert_log_line",
	name = "insert_log_line",
	description = "Insert a log line at end of Settings.update()",
	args = {
		{ name = "file", required = true, inputKind = "file" },
	},
	postExecution = {},
	apply = function(c)
		c:insert({
			path = c.args.file,
			language = "dart",
			query = [[
(class_definition
  name: (identifier) @className
  body: (class_body
    (method_signature
      (function_signature
        name: (identifier) @methodName))
    (function_body
      (block) @body))
  (#eq? @className "Settings")
  (#eq? @methodName "update"))
]],
			capture = "body",
			anchor = "end",
			text = "    print('codemod');\n",
		})
	end,
}
