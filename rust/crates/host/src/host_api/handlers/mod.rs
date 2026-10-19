//! Host API `c:*` method handlers (`#[host_method]`).

mod edits;
mod files;
mod helpers;
mod log;
mod queries;
mod template_methods;

pub(crate) use files::do_resolve;
pub(crate) use helpers::post_exec_cmd;
pub(crate) use log::do_log;
