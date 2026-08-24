return {
	id = "scaffold_feature",
	name = "scaffold_feature",
	description = "Composed scaffold: create repository, patch counter + app, delete legacy file.",
	args = {
		{ name = "className", required = true, inputKind = "symbol", help = "PascalCase class name for the repository template" },
		{ name = "fieldName", required = true, inputKind = "symbol", help = "Field to insert on Counter (use tickCount or label)" },
	},
	apply = function(c)
		c:run("create_repository")
		c:run("patch_counter")
		c:run("patch_app")
		c:delete({ path = "lib/legacy/stale.dart", ifMissing = "skip" })
	end,
}
