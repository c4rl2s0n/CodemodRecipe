return {
	id = "example.entity",
	description = "Entity files under src/entities",
	discover = function(c)
		local paths = c:glob("src/entities/*.dart")
		local locators = {}
		for i, path in ipairs(paths) do
			locators[i] = { path = path }
		end
		return locators
	end,
	identify = function(_c, locator)
		return locator.path ~= nil and locator.path:find("src/entities/", 1, true) ~= nil
	end,
	parse = function(c, locator)
		local src = c:read(locator.path)
		local fields = {}
		for name in src:gmatch("final%s+String%s+(%w+)") do
			fields[#fields + 1] = c:component("example.property"):parse({
				path = locator.path,
				field = name,
			})
		end
		return {
			id = c:stem(locator.path),
			name = c:stem(locator.path),
			path = locator.path,
			fields = fields,
		}
	end,
	verify = function(_c, instance)
		local findings = {}
		if #instance.parsed.fields == 0 then
			findings[#findings + 1] = {
				severity = "warning",
				code = "entity.no_fields",
				message = "entity has no String fields",
				instanceId = instance.id,
			}
		end
		return findings
	end,
	operations = {
		addField = {
			apply = function(c, args, identity)
				local fieldName = args.name
				if not fieldName or fieldName == "" then
					error("addField requires args.name")
				end
				c:insert({
					path = identity.parsed.path,
					language = "dart",
					query = "(class_definition) @class",
					capture = "class",
					anchor = "end",
					text = "\n  final String " .. fieldName .. " = '';",
				})
			end,
		},
	},
}
