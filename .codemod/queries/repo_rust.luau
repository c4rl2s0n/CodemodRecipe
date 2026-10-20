return {
	id = "repo_rust",
	queries = {
		-- Variants of the HostCommand enum (name only; serde rename paired by span).
		host_command_variants = [[
(enum_item
  name: (type_identifier) @enum
  body: (enum_variant_list
    (enum_variant
      name: (identifier) @variant))
  (#eq? @enum "HostCommand"))
]],
		-- serde rename string literals on HostCommand variants.
		host_command_renames = [[
(enum_item
  name: (type_identifier) @enum
  body: (enum_variant_list
    (attribute_item
      (attribute
        (identifier) @attr
        (token_tree
          (identifier) @key
          (string_literal
            (string_content) @rename)))))
  (#eq? @enum "HostCommand")
  (#eq? @attr "serde")
  (#eq? @key "rename"))
]],
		-- Dispatch arms: HostCommand::Variant => ... (unit variants); whole arm for coverage.
		host_command_dispatch_unit = [[
(match_arm
  pattern: (match_pattern
    (scoped_identifier
      path: (identifier) @path
      name: (identifier) @dispatchVariant))
  (#eq? @path "HostCommand")) @dispatchArm
]],
		-- Dispatch arms: HostCommand::Variant { .. } => ... (struct variants)
		host_command_dispatch_struct = [[
(match_arm
  pattern: (match_pattern
    (struct_pattern
      type: (scoped_type_identifier
        path: (identifier) @path
        name: (type_identifier) @dispatchVariant)))
  (#eq? @path "HostCommand")) @dispatchArm
]],
		-- Match arms with a string-literal pattern (method/tool routing): name + arm value.
		string_match_arms = [[
(match_arm
  pattern: (match_pattern
    (string_literal
      (string_content) @stringArmName))
  value: (_) @stringArmValue)
]],
		-- All string literal contents (pair "name" keys with values inside a span).
		string_literals = [[
(string_literal
  (string_content) @str)
]],
		-- HostCommand variant references in expression position (unit variants).
		-- NOTE: predicates must nest inside the pattern's parens — a trailing
		-- top-level predicate is silently ignored by the query engine.
		host_command_expr_unit = [[
(scoped_identifier
  path: (identifier) @path
  name: (identifier) @hcVariant
  (#eq? @path "HostCommand"))
]],
		-- HostCommand variant references in expression position (struct variants).
		host_command_expr_struct = [[
(struct_expression
  (scoped_type_identifier
    path: (identifier) @path
    name: (type_identifier) @hcVariant)
  (#eq? @path "HostCommand"))
]],
		-- Function items (name + whole-function span) for helper-indirection analysis.
		function_items = [[
(function_item
  name: (identifier) @fnName) @fn
]],
		-- Direct function calls (callee identifier).
		call_exprs = [[
(call_expression
  function: (identifier) @callee)
]],
		-- HostCommand enum body (for createCommand insert-at-end).
		host_command_enum_body = [[
(enum_item
  name: (type_identifier) @enum
  body: (enum_variant_list) @body
  (#eq? @enum "HostCommand"))
]],
		-- handle_command match arms body (dispatch insert-at-end).
		host_command_dispatch_match = [[
(function_item
  name: (identifier) @fn
  body: (block
    (expression_statement
      (match_expression
        (match_block) @match)))
  (#eq? @fn "handle_command"))
]],
		-- Module / barrel declarations (mod name; / pub mod name;).
		mod_items = [[
(mod_item
  name: (identifier) @modName) @modItem
]],
		-- Top-level use declarations (pub use and private).
		use_declarations = [[
(use_declaration) @useItem
]],
		-- Named type definitions (struct / enum / type alias) with whole-item span.
		type_items = [[
(struct_item
  name: (type_identifier) @typeName) @typeItem

(enum_item
  name: (type_identifier) @typeName) @typeItem

(type_item
  name: (type_identifier) @typeName) @typeItem
]],
		-- Pub function items (signature name + whole fn span).
		pub_functions = [[
(function_item
  (visibility_modifier)
  name: (identifier) @fnName) @fn
]],
		-- All function items (pub + private) for span deepen.
		all_functions = [[
(function_item
  name: (identifier) @fnName) @fn
]],
		-- Impl blocks with type name.
		impl_items = [[
(impl_item
  type: (type_identifier) @implType) @implItem
]],
	},
}
