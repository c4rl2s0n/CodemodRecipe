return {
	id = "insert_log_line",
	name = "duplicate_insert_log_line",
	description = "Duplicate id for registry tests",
	args = {
		{ name = "file", required = true, inputKind = "file" },
	},
	postExecution = {},
	apply = function(c)
		c:insert({
			path = c.args.file,
			query = "(identifier) @x",
			capture = "x",
			anchor = "start",
			text = "",
		})
	end,
}
