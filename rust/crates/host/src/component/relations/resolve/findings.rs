//! Relation findings helpers (severity + code + message).

use crate::component::types::ComponentFinding;

pub(super) fn relation_finding(
    code: &str,
    message: &str,
    instance_id: &str,
    locator: Option<serde_json::Value>,
) -> ComponentFinding {
    ComponentFinding {
        severity: "warning".to_string(),
        code: code.to_string(),
        message: message.to_string(),
        instance_id: Some(instance_id.to_string()),
        locator,
    }
}

pub(super) fn relation_info_finding(
    code: &str,
    message: &str,
    instance_id: &str,
    locator: Option<serde_json::Value>,
) -> ComponentFinding {
    ComponentFinding {
        severity: "info".to_string(),
        code: code.to_string(),
        message: message.to_string(),
        instance_id: Some(instance_id.to_string()),
        locator,
    }
}
