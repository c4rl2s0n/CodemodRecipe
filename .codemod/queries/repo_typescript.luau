return {
	id = "repo_typescript",
	queries = {
		-- All `command: 'x'` literal members of object types inside any union.
		-- (union_type nests left, so alias-anchored matching only sees the last
		-- member; filter by host_command_alias_decl span containment instead.)
		host_command_union_members = [[
(union_type
  (object_type
    (property_signature
      name: (property_identifier) @prop
      type: (type_annotation
        (literal_type
          (string
            (string_fragment) @tsCommand)))))
  (#eq? @prop "command"))
]],
		-- Span of the `export type HostCommand = ...` declaration.
		host_command_alias_decl = [[
((type_alias_declaration
  name: (type_identifier) @alias) @aliasDecl
  (#eq? @alias "HostCommand"))
]],
		-- Entries of the COMMANDS const map: key → 'codemodRecipe.*' id.
		commands_constants = [[
(lexical_declaration
  (variable_declarator
    name: (identifier) @constName
    value: (as_expression
      (object
        (pair
          key: (property_identifier) @constKey
          value: (string
            (string_fragment) @commandId)))))
  (#eq? @constName "COMMANDS"))
]],
		-- Registration sites: vscode.commands.registerCommand(COMMANDS.x, ...).
		command_registrations = [[
(call_expression
  function: (member_expression
    property: (property_identifier) @fn)
  arguments: (arguments
    (member_expression
      object: (identifier) @obj
      property: (property_identifier) @registeredKey))
  (#eq? @fn "registerCommand")
  (#eq? @obj "COMMANDS"))
]],
		-- Registration sites bypassing the COMMANDS const with a string literal.
		command_registrations_literal = [[
(call_expression
  function: (member_expression
    property: (property_identifier) @fn)
  arguments: (arguments
    (string
      (string_fragment) @literalId))
  (#eq? @fn "registerCommand"))
]],
		-- `command: 'x'` payload pairs in send*/sendPersistent/sendOneShot call arguments
		-- (anchored on the call so spawn options like `{ command: 'cargo' }` don't match).
		-- Two shapes: plain calls, and awaited generic calls where this grammar wraps
		-- the member_expression in an await_expression inside the function field.
		bridge_command_literals = [[
(call_expression
  function: (member_expression
    property: (property_identifier) @sendFn)
  arguments: (arguments
    (object
      (pair
        key: (property_identifier) @key
        value: (string
          (string_fragment) @commandLiteral))))
  (#eq? @key "command"))

(call_expression
  function: (await_expression
    (member_expression
      property: (property_identifier) @sendFn))
  arguments: (arguments
    (object
      (pair
        key: (property_identifier) @key
        value: (string
          (string_fragment) @commandLiteral))))
  (#eq? @key "command"))
]],
		-- Class methods (name + whole-method span) for containment mapping.
		bridge_methods = [[
(method_definition
  name: (property_identifier) @methodName) @method
]],
		-- registerCommand(COMMANDS.x, handler) with the handler node (kind + span).
		command_registration_handlers = [[
(call_expression
  function: (member_expression
    property: (property_identifier) @fn)
  arguments: (arguments
    (member_expression
      object: (identifier) @obj
      property: (property_identifier) @registeredKey)
    .
    (_) @handler)
  (#eq? @fn "registerCommand")
  (#eq? @obj "COMMANDS"))
]],
		-- All call expression callees (text classifies bridge / this-local / external).
		all_calls = [[
(call_expression
  function: (_) @callee)
]],
		-- HostCommand type alias body (union) for insert-at-end.
		host_command_union = [[
((type_alias_declaration
  name: (type_identifier) @alias
  value: (type_annotation
    (_) @union))
  (#eq? @alias "HostCommand"))
]],
		-- COMMANDS const object for insert-at-end.
		commands_const_object = [[
(lexical_declaration
  (variable_declarator
    name: (identifier) @constName
    value: (as_expression
      (object) @commandsObj))
  (#eq? @constName "COMMANDS"))
]],
		-- Top-level export statements (whole statement span).
		export_statements = [[
(export_statement) @export
]],
		-- Exported lexical declarations: export const/let/var Name = ...
		export_lexical = [[
(export_statement
  (lexical_declaration
    (variable_declarator
      name: (identifier) @exportName))) @export
]],
		-- export function Name
		export_function = [[
(export_statement
  (function_declaration
    name: (identifier) @exportName)) @export
]],
		-- export class Name
		export_class = [[
(export_statement
  (class_declaration
    name: (type_identifier) @exportName)) @export
]],
		-- export type / interface Name
		export_type = [[
(export_statement
  (type_alias_declaration
    name: (type_identifier) @exportName)) @export

(export_statement
  (interface_declaration
    name: (type_identifier) @exportName)) @export
]],
		-- Object pair keys inside as-const maps (message consts etc.).
		as_const_object_keys = [[
(lexical_declaration
  (variable_declarator
    name: (identifier) @constName
    value: (as_expression
      (object
        (pair
          key: (property_identifier) @key)))))
]],
	},
}
