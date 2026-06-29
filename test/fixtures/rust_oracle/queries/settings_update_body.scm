(class_declaration
  name: (identifier) @className
  body: (class_body
    (class_member
      (method_signature
        (function_signature
          name: (identifier) @methodName))
      (function_body
        (block) @body)))
  (#eq? @className "Settings")
  (#eq? @methodName "update"))
