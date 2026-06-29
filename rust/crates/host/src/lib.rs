pub mod config;
pub mod diff;
pub mod dispatch;
pub mod patch_selector;
pub mod path_sandbox;
pub mod post_execution;
pub mod preview_token;
pub mod protocol;
pub mod registry;
pub mod runner;
pub mod template;

pub const RESULT_BEGIN: &str = "__CODEMOD_RESULT_BEGIN__";
pub const RESULT_END: &str = "__CODEMOD_RESULT_END__";
