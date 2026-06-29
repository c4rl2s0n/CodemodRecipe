use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(tag = "command")]
#[allow(dead_code)] // fields accepted for protocol compatibility; not all used in v1
pub enum HostCommand {
    #[serde(rename = "list")]
    List,
    #[serde(rename = "reload")]
    Reload,
    #[serde(rename = "validate")]
    Validate,
    #[serde(rename = "describe")]
    Describe { recipe: String },
    #[serde(rename = "preview")]
    Preview {
        recipe: String,
        args: std::collections::BTreeMap<String, String>,
        #[serde(default, rename = "snippetLines")]
        snippet_lines: Option<u32>,
    },
    #[serde(rename = "apply")]
    Apply {
        recipe: String,
        args: std::collections::BTreeMap<String, String>,
        #[serde(rename = "previewToken")]
        preview_token: String,
        selection: serde_json::Value,
    },
    #[serde(rename = "diff")]
    Diff {
        recipe: String,
        args: std::collections::BTreeMap<String, String>,
        path: String,
    },
    #[serde(rename = "generateAstPath")]
    GenerateAstPath { path: String, offset: u64 },
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipeArg {
    pub name: String,
    pub abbr: Option<String>,
    pub help: Option<String>,
    pub required: bool,
    pub defaults_to: Option<String>,
    pub input_kind: String,
    pub options: Vec<String>,
    pub allow_custom_value: bool,
    pub context_key: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RecipeSchema {
    pub id: String,
    pub name: String,
    pub description: String,
    pub args: Vec<RecipeArg>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RecipeDiagnostic {
    pub severity: &'static str,
    pub code: &'static str,
    pub message: String,
    pub sources: Vec<DiagnosticSource>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DiagnosticSource {
    pub file: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
pub struct RecipeCatalogResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipes: Option<Vec<RecipeSchema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<RecipeDiagnostic>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DescribeResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<RecipeSchema>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PatchInfo {
    pub index: usize,
    pub offset: usize,
    pub length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replacement_preview: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilePreview {
    pub path: String,
    pub kind: &'static str,
    pub is_new: bool,
    pub skipped: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    pub patches: Vec<PatchInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snippet: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PreviewResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FilePreview>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ApplyResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DiffResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<FilePreview>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ValidateResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<RecipeDiagnostic>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct AstPathResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

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
                assert_eq!(recipe, "r");
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
        };
        let value = serde_json::to_value(&resp).unwrap();
        assert_eq!(value["previewToken"], "abc");
    }
}
