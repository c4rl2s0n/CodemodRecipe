return {
	id = "jinja.casing.showcase",
	name = "showcase_casing",
	description = "Demonstrates Jinja casing filters and map lookups",
	args = {
		{ name = "name", required = true, inputKind = "symbol" },
		{ name = "fieldKey", required = true, inputKind = "symbol" },
	},
	maps = {
		field_kind = {
			tickCount = "int",
			label = "String",
		},
	},
	apply = function(c)
		c:create({
			path = c:applyTemplate({ template = "lib/generated/{{ name | snake_case }}_casing.dart" }),
			content = c:applyTemplate({ template = [[
// snake={{ name | snake_case }}
// camel={{ name | camel_case }}
// pascal={{ name | pascal_case }}
// lower={{ name | lower }}
// upper={{ name | upper }}
// screaming={{ name | screaming_snake }}
// kebab={{ name | kebab_case }}
// mapFilter={{ fieldKey | map('field_kind') }}
// mapContext={{ map.field_kind[fieldKey] }}
class {{ name | pascal_case }}Casing {}
]] }),
			ifExists = "fail",
		})
	end,
}
