return {
	id = "jinja.step_if.orchestrator",
	name = "step_if_orchestrator",
	description = "Demonstrates step-level if / ifNot gates on recipe composition and create.",
	args = {
		{ name = "className", required = true },
		{ name = "includeExtra", defaultsTo = "false" },
		{ name = "markerFile", defaultsTo = "lib/generated/marker.dart" },
	},
	apply = function(c, args)
		local function pathExists(path)
			local ok = pcall(function()
				c:read(path)
			end)
			return ok
		end

		c:create({
			path = c:template({ inline = "lib/generated/{{ className | snake_case }}_base.dart" }),
			content = c:template({ inline = "class {{ className | pascal_case }}Base {}\n" }),
		})
		if args.includeExtra == "true" then
			c:run("jinja.step_if.child", { className = args.className })
		end
		local markerFile = args.markerFile or "lib/generated/marker.dart"
		if not pathExists(markerFile) then
			c:create({
				path = "lib/generated/skipped_when_marker.dart",
				content = "// only when markerFile is absent\n",
			})
		end
	end,
}
