return {
	id = "patch_counter",
	name = "patch_counter",
	description = "Insert a typed counter field before increment()",
	args = {
		{ name = "className", required = true, inputKind = "symbol" },
		{ name = "fieldName", required = true, inputKind = "symbol" },
	},
	maps = {
		field_kind = {
			tickCount = "int",
			label = "String",
		},
	},
	apply = function(c)
		local query = c:template({ inline = [[
(class_definition
  name: (identifier) @className
  body: (class_body
    (method_signature
      (function_signature
        name: (identifier) @methodName)) @member)
  (#eq? @className "{{className}}")
  (#eq? @methodName "increment"))
]] })
		c:insert({
			path = "lib/counter/counter.dart",
			language = "dart",
			query = query,
			capture = "member",
			anchor = "start",
			text = c:template({ inline = "  final {{ fieldName | map('field_kind') }} {{ fieldName | camel_case }} = 0;\n\n" }),
		})
	end,
}
