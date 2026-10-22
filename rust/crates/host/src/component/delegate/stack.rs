//! Component call-stack cycle guard for nested `c:component` delegates.

use crate::component::entry;
use crate::host_api::context::SharedHost;

pub(super) fn push_component(host: &SharedHost, id: &str) -> Result<(), String> {
    entry::with_component_host(host, |h| h.push_component(id))
}

pub(super) fn pop_stack(host: &SharedHost, id: &str) {
    let _ = entry::with_component_host(host, |h| {
        h.pop_component(id);
        Ok(())
    });
}
