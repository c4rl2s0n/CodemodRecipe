return {
	id = "flutter",
	queries = {
		model_class = [[
(class_definition
  name: (identifier) @className)
]],
		model_field = [[
(class_definition
  body: (class_body
    (declaration
      (initialized_identifier_list
        (initialized_identifier
          (identifier) @fieldName)))))
]],
		store_body = [[
(class_definition
  body: (class_body) @classBody)
]],
		model_method = [[
(class_definition
  body: (class_body
    (method_signature
      (function_signature name: (identifier) @methodName)) @method))
]],
	},
}
