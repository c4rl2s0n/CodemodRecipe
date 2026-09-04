return {
	id = "insert_log_line_scm",
	name = "insert_log_line_scm",
	description = "Insert a log line using an external .scm query file",
	args = {
		{ name = "file", required = true, inputKind = "file" },
	},
	postExecution = {},
	apply = function(c, args)
		c:insert({
			path = args.file,
			language = "dart",
			query = "queries/settings_update_body.scm",
			capture = "body",
			anchor = "end",
			text = "    print('codemod');\n",
		})
	end,
}
