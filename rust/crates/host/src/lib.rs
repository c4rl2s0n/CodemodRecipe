pub mod component;
pub mod config;
pub mod diagnostics;
pub mod edit_session;
pub mod explorer;
pub mod host_api;
pub mod protocol;
pub mod query;
pub mod recipe;
pub mod sandbox;
pub mod template;
pub mod testing;

/// Compat: preview lives under `edit_session::preview`.
pub use edit_session::preview;
/// Compat: lua runtime lives under `host_api::lua_runtime`.
pub use host_api::lua_runtime;

pub const RESULT_BEGIN: &str = "__CODEMOD_RESULT_BEGIN__";
pub const RESULT_END: &str = "__CODEMOD_RESULT_END__";
