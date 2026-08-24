return {
  id = "ClassProperty",
  description = "A class field: type, name, optional initial value",
  fields = {
    { name = "type", inputKind = "text", required = true },
    { name = "name", inputKind = "text", required = true },
    { name = "initialValue", inputKind = "text", required = false },
  },
}
