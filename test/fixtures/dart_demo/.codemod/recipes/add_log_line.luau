return {
	id = "add_log_line",
	name = "add_log_line",
	description = "Inserts a log line at the end of a method",
	args = {
		{ name = "file", required = true, inputKind = "file" },
		{ name = "className", required = true, inputKind = "symbol" },
		{ name = "methodName", required = true, inputKind = "symbol" },
	},
	postExecution = {},
	apply = function(c, args)
		local query = c:template({ inline = [[
(class_definition
  name: (identifier) @className
  body: (class_body
    (method_signature
      (function_signature
        name: (identifier) @methodName))
    (function_body
      (block) @body))
  (#eq? @className "{{className}}")
  (#eq? @methodName "{{methodName}}"))
]] })
		c:insert({
			path = args.file,
			language = "dart",
			query = query,
			capture = "body",
			anchor = "end",
			text = "    print('codemod');\n",
		})
	end,
}
