return {
	id = "jinja.path.showcase",
	name = "showcase_path",
	description = "Demonstrates Jinja path filters (parent, basename, stem)",
	args = {
		{ name = "featureDir", required = true, inputKind = "directory" },
		{ name = "file", required = true, inputKind = "file" },
	},
	apply = function(c, args)
		c:create({
			path = c:template({ inline = "lib/generated/{{ featureDir | basename }}_path.dart" }),
			content = c:template({ inline = [[
// parent={{ featureDir | parent }}
// basename={{ featureDir | basename }}
// parentBasename={{ featureDir | parent | basename }}
// stem={{ file | stem }}
class {{ featureDir | basename | pascal_case }}Path {}
]] }),
			ifExists = "fail",
		})
	end,
}
