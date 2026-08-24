return {
	id = "jinja.path.showcase",
	name = "showcase_path",
	description = "Demonstrates Jinja path filters (parent, basename, stem)",
	args = {
		{ name = "featureDir", required = true, inputKind = "directory" },
		{ name = "file", required = true, inputKind = "file" },
	},
	apply = function(c)
		c:create({
			path = c:applyTemplate({ template = "lib/generated/{{ featureDir | basename }}_path.dart" }),
			content = c:applyTemplate({ template = [[
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
