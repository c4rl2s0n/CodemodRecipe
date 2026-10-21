pub mod command;
pub mod dispatch;
pub mod keys;
pub mod mcp_inventory;
pub mod responses;

pub use command::HostCommand;
pub use responses::*;

pub use crate::component::types::{
    ComponentFinding, ComponentInstance, ComponentInvokeResponse, ComponentScanResponse,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_preview_command_with_camel_case_fields() {
        let json =
            r#"{"command":"preview","recipe":"r","args":{"file":"a.dart"},"snippetLines":3}"#;
        let cmd: HostCommand = serde_json::from_str(json).unwrap();
        match cmd {
            HostCommand::Preview {
                recipe,
                snippet_lines,
                ..
            } => {
                assert_eq!(recipe.as_deref(), Some("r"));
                assert_eq!(snippet_lines, Some(3));
            }
            _ => panic!("expected preview"),
        }
    }

    #[test]
    fn serializes_preview_response_with_camel_case_fields() {
        let resp = PreviewResponse {
            ok: true,
            error: None,
            recipe: Some("r".to_string()),
            preview_token: Some("abc".to_string()),
            files: None,
            transcript: None,
            logs: None,
            apply_may_diverge: None,
            diagnostics: None,
        };
        let value = serde_json::to_value(&resp).unwrap();
        assert_eq!(value["previewToken"], "abc");
    }
}
