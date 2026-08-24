return {
	id = "delete_legacy",
	args = {
		{ name = "legacyPath", required = true },
	},
	apply = function(c)
		c:delete({
			path = c:applyTemplate({ template = "{{legacyPath}}" }),
			ifMissing = "fail",
		})
	end,
}
