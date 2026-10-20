use schemars::schema::RootSchema;

/// Schema-backed type used by the host API surface generator (inputs and returns).
pub trait HostType: 'static + Send {
    /// Rust/Luau name of the type (e.g. `InsertSpec`, `QueryHit`).
    const NAME: &'static str;
    /// JSON schema used by the surface generator.
    fn schema() -> RootSchema;
}

/// Metadata trait implemented by every Luau-facing host API **input** spec type.
///
/// The `#[derive(LuaSpec)]` macro provides the implementation for simple
/// table specs; complex types (e.g. `QuerySpec`, `ReadSpec`, `Env`) implement
/// it manually. Return-only shapes implement [`HostType`] without this trait.
pub trait HostSpec: HostType + mlua::FromLua + Sized {}
