return {
	id = "patch_app",
	name = "patch_app",
	description = "Insert a log line in App.run()",
	apply = function(c)
		c:insert({
			path = "lib/app.dart",
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
  (#eq? @className "App")
  (#eq? @methodName "run"))
]],
			capture = "body",
			anchor = "end",
			text = "    print('scaffold');\n",
		})
	end,
}
