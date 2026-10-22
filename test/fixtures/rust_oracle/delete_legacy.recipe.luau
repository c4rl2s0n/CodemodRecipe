return {
	id = "delete_legacy",
	args = {
		{ name = "legacyPath", required = true },
	},
	apply = function(c, args)
		c:delete({
			path = c:template({ inline = "{{legacyPath}}" }),
			ifMissing = "fail",
		})
	end,
}
