pub mod component;
pub mod config;
pub mod diagnostics;
pub mod edit_session;
pub mod explorer;
pub mod host_api;
pub mod lua_runtime;
pub mod preview;
pub mod protocol;
pub mod query;
pub mod recipe;
pub mod sandbox;
pub mod template;
pub mod testing;

pub const RESULT_BEGIN: &str = "__CODEMOD_RESULT_BEGIN__";
pub const RESULT_END: &str = "__CODEMOD_RESULT_END__";
