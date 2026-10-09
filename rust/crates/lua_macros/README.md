# codemod_recipe_lua_macros

Proc-macro support for the codemod-recipe Luau host API.

Provides two macros that keep the runtime binding, the LSP type stub, and the
VS Code editor surface in sync from a single source of truth:

- `#[derive(LuaSpec)]` — generates `mlua::FromLua` and `HostSpec` trait
  implementations for simple Luau table specs.
- `#[host_method(...)]` — marks a handler function as a `c:` method and emits
  both the runtime registry entry and the static surface metadata used by the
  generator.

## Adding a new `c:` method

1. **Define the spec struct** in `rust/crates/lua/src/host_api/specs.rs`:

   ```rust
   #[derive(Debug, Clone, PartialEq, Eq, JsonSchema, LuaSpec)]
   #[serde(rename_all = "camelCase")]
   pub struct MySpec {
       pub path: String,
       pub query: QuerySpec,
   }
   ```

   The Rust struct name is also the generated Luau type name. Use
   `#[serde(rename_all = "camelCase")]` / `#[serde(rename = "...")]` so the
   Lua table keys match the JSON schema.

2. **Implement the handler** in `rust/crates/host/src/host_api/handlers.rs` and
   annotate it:

   ```rust
   #[host_method(
       name = "myMethod",
       description = "What myMethod does.",
       spec = MySpec,
       return_type = Nil,
   )]
   pub(crate) fn do_my_method(
       host: &SharedHost,
       _lua: &Lua,
       spec: MySpec,
   ) -> Result<Value, String> {
       // ...
   }
   ```

   For methods that take a string id, use `string_id` instead of `spec`. For the
   assertion helper, use `assert`. For `run`, use `run` with `recursive = true`.

3. **Regenerate the editor artifacts** (or run the full product build):

   ```bash
   ./scripts/generate-host-api-surface.sh   # stubs + hostApiSurface.ts only
   ./build.sh                              # surface + release host + MCP + VSIX
   ```

   The generate script updates both `vscode_extension/stubs/codemod-context.d.luau`
   and `vscode_extension/src/extension/language/hostApiSurface.ts` from the same
   Rust definitions.

## How it works

`#[derive(LuaSpec)]` inspects the struct and any `#[serde(rename_all = ...)]` /
`#[serde(rename = ...)]` attributes, then produces a `FromLua` impl that reads the
matching camelCase (or otherwise renamed) keys from the Luau table. The Luau
type alias name is always `HostSpec::NAME` (the Rust struct name).

`#[host_method(...)]` produces two `linkme` distributed-slice entries: one in
`HOST_API_SURFACE` (the contract metadata) and one in `HOST_METHODS` (the runtime
registration closure). The generator binary in `rust/crates/host/src/bin/`
iterates `HOST_API_SURFACE` and the spec schemas to emit the Luau and TypeScript
files.
