//! Component call-stack cycle guard for nested `c:component` delegates.

use crate::host_api::context::SharedHost;

pub(super) fn push_component(host: &SharedHost, id: &str) -> Result<(), String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    if inner.component_stack.iter().any(|c| c == id) {
        return Err(format!("component cycle detected: {id}"));
    }
    inner.component_stack.push(id.to_string());
    Ok(())
}

pub(super) fn pop_stack(host: &SharedHost, id: &str) {
    if let Ok(mut inner) = host.lock() {
        if inner.component_stack.last().map(|s| s.as_str()) == Some(id) {
            inner.component_stack.pop();
        }
    }
}
