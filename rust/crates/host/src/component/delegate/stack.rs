//! Component call-stack cycle guard for nested `c:component` delegates.

use crate::host_api::context::SharedHost;
use crate::host_api::facets::ComponentSession;

pub(super) fn push_component(host: &SharedHost, id: &str) -> Result<(), String> {
    ComponentSession::with_host(host, |mut session| session.push_component(id))
}

pub(super) fn pop_stack(host: &SharedHost, id: &str) {
    let _ = ComponentSession::with_host(host, |mut session| {
        session.pop_component(id);
        Ok(())
    });
}
