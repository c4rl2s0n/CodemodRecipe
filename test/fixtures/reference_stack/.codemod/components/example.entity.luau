return {
	id = "example.entity",
	description = "Entity files under src/entities",
	identify = function(c)
		local out = {}
		for _, row in ipairs(c:matchFiles("src/entities/{name}.dart")) do
			out[#out + 1] = { path = row.path, name = row.name }
		end
		return out
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
			id = locator.name or c:filename(locator.path),
			name = locator.name or c:filename(locator.path),
			path = locator.path,
			fields = fields,
		}
	end,
	verify = function(_c, instance)
		local findings = {}
		if #instance.node.fields == 0 then
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
			apply = function(c, args, target)
				local fieldName = args.name
				if not fieldName or fieldName == "" then
					error("addField requires args.name")
				end
				c:insert({
					path = target.node.path,
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
