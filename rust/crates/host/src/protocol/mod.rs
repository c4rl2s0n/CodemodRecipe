pub mod dispatch;
pub mod keys;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
#[serde(tag = "command")]
pub enum HostCommand {
    #[serde(rename = "list")]
    List,
    #[serde(rename = "reload")]
    Reload,
    #[serde(rename = "validate")]
    Validate {
        #[serde(default)]
        recipe: Option<String>,
    },
    #[serde(rename = "describe")]
    Describe { recipe: String },
    #[serde(rename = "preview")]
    Preview {
        #[serde(default)]
        recipe: Option<String>,
        #[serde(default, rename = "inlineRecipe")]
        inline_recipe: Option<serde_json::Value>,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
        #[serde(default, rename = "snippetLines")]
        snippet_lines: Option<u32>,
    },
    #[serde(rename = "apply")]
    Apply {
        #[serde(default)]
        recipe: Option<String>,
        #[serde(default, rename = "inlineRecipe")]
        inline_recipe: Option<serde_json::Value>,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
        #[serde(rename = "previewToken")]
        preview_token: String,
        #[serde(default)]
        selection: serde_json::Value,
    },
    #[serde(rename = "diff")]
    Diff {
        #[serde(default)]
        recipe: Option<String>,
        #[serde(default, rename = "inlineRecipe")]
        inline_recipe: Option<serde_json::Value>,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
        path: String,
    },
    #[serde(rename = "generateAstPath")]
    GenerateAstPath { path: String, offset: u64 },
    #[serde(rename = "bootstrap")]
    Bootstrap {
        #[serde(default)]
        force: bool,
        #[serde(default, rename = "editPolicy")]
        edit_policy: Option<String>,
        #[serde(default)]
        companions: Vec<String>,
    },
    #[serde(rename = "deriveArgs")]
    DeriveArgs {
        recipe: String,
        source: String,
        #[serde(default)]
        language: Option<String>,
        #[serde(default)]
        path: Option<String>,
        #[serde(default, rename = "cursorOffset")]
        cursor_offset: u64,
        #[serde(default, rename = "selectionStart")]
        selection_start: u64,
        #[serde(default, rename = "selectionEnd")]
        selection_end: u64,
        #[serde(default)]
        context: std::collections::BTreeMap<String, String>,
    },
    #[serde(rename = "filterExplorerRecipes")]
    FilterExplorerRecipes { path: String, kind: String },
    #[serde(rename = "dumpAst")]
    DumpAst {
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        language: Option<String>,
        #[serde(default = "default_true", rename = "namedOnly")]
        named_only: bool,
    },
    #[serde(rename = "debugQuery")]
    DebugQuery {
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        language: Option<String>,
        query: String,
        #[serde(default = "default_true")]
        instrument: bool,
        #[serde(default, rename = "includeSexp")]
        include_sexp: bool,
    },
    #[serde(rename = "generateQuery")]
    GenerateQuery {
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        language: Option<String>,
        start: u64,
        #[serde(default)]
        end: u64,
        #[serde(default, rename = "includeTextPredicates")]
        include_text_predicates: bool,
        #[serde(default, rename = "captureLeaf")]
        capture_leaf: Option<String>,
        #[serde(default, rename = "maxDepth")]
        max_depth: Option<u64>,
    },
    #[serde(rename = "resolveStaticPath")]
    ResolveStaticPath { template: String },
    /// Resolve `@capture` names from an inline query or registered query id.
    #[serde(rename = "queryCaptures")]
    QueryCaptures {
        #[serde(default)]
        query: Option<String>,
        #[serde(default, rename = "queryId")]
        query_id: Option<String>,
    },
    #[serde(rename = "componentScan")]
    ComponentScan,
    #[serde(rename = "componentIncrementalScan")]
    ComponentIncrementalScan {
        #[serde(default, rename = "dirtyPaths")]
        dirty_paths: Vec<String>,
        #[serde(default, rename = "detectChanges")]
        detect_changes: bool,
        #[serde(default)]
        force: bool,
    },
    #[serde(rename = "componentListInstances")]
    ComponentListInstances,
    #[serde(rename = "componentCoverage")]
    ComponentCoverage,
    /// Render the relations graph as mermaid text (`level`: `kinds` default, or
    /// `instances`). Scan-on-demand when no scan cache exists.
    #[serde(rename = "componentGraph")]
    ComponentGraph {
        #[serde(default)]
        level: Option<String>,
    },
    #[serde(rename = "componentInvokeOperation")]
    ComponentInvokeOperation {
        #[serde(rename = "instanceId")]
        instance_id: String,
        operation: String,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
        #[serde(default)]
        preview: bool,
    },
    #[serde(rename = "componentScaffold")]
    ComponentScaffold {
        #[serde(rename = "componentId")]
        component_id: String,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
        #[serde(default)]
        preview: bool,
    },
    /// Dry-run or apply a single component hook (`identify` / `parse` / `verify` /
    /// `toSummary` / `scaffold` / `operation`). MCP exposes separate tools that set
    /// `preview` true vs false for allowlisting.
    #[serde(rename = "componentInvokeHook")]
    ComponentInvokeHook {
        #[serde(rename = "componentId")]
        component_id: String,
        hook: String,
        #[serde(default = "default_true")]
        preview: bool,
        #[serde(default)]
        locator: Option<serde_json::Value>,
        #[serde(default)]
        instance: Option<serde_json::Value>,
        #[serde(default)]
        operation: Option<String>,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
    },
    #[serde(rename = "describeComponent")]
    DescribeComponent {
        #[serde(rename = "componentId")]
        component_id: String,
    },
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipeArg {
    pub name: String,
    pub abbr: Option<String>,
    pub help: Option<String>,
    pub required: bool,
    pub defaults_to: Option<String>,
    pub input_kind: serde_json::Value,
    pub options: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options_from_map: Option<String>,
    pub allow_custom_value: bool,
    pub context_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<serde_json::Value>,
    /// Nested recipe ids that contribute this unbound arg (empty when parent-declared).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub from_recipes: Vec<String>,
    /// Resolved fields when `inputKind` is a named type or list of a named type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<RecipeArg>>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DeriveArgsResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<std::collections::BTreeMap<String, serde_json::Value>>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipeSchema {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Workspace-relative path to the recipe Luau module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    pub args: Vec<RecipeArg>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub explorer_menu: Option<Vec<ExplorerMenuEntrySchema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_execution: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TypeSchema {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub fields: Vec<RecipeArg>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExplorerMenuEntrySchema {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "if")]
    pub if_expr: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub args: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExplorerRecipeMatch {
    pub recipe_id: String,
    pub args: std::collections::BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FilterExplorerRecipesResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matches: Option<Vec<ExplorerRecipeMatch>>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RecipeDiagnostic {
    pub severity: &'static str,
    pub code: &'static str,
    pub message: String,
    pub sources: Vec<DiagnosticSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_recipe: Option<String>,
}

impl RecipeDiagnostic {
    pub fn simple(
        severity: &'static str,
        code: &'static str,
        message: String,
        sources: Vec<DiagnosticSource>,
    ) -> Self {
        Self {
            severity,
            code,
            message,
            sources,
            hint: None,
            related_recipe: None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct DiagnosticSource {
    pub file: String,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueryCapturesResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub captures: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentOperationSchema {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub args: Vec<RecipeArg>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentSchema {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub operations: Vec<String>,
    pub has_scaffold: bool,
    pub is_root: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct StackSchema {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub components: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub codebase: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DescribeComponentResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<ComponentDetailSchema>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentGraphResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mermaid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind_graph: Option<Vec<crate::component::KindEdge>>,
    /// Resolved instance-level edges (only for `level = "instances"`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edges: Option<Vec<crate::component::ComponentEdge>>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDetailSchema {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub has_identify: bool,
    pub has_verify: bool,
    pub has_relate: bool,
    pub has_to_summary: bool,
    pub has_scaffold: bool,
    pub is_root: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_file: Option<String>,
    pub scaffold_args: Vec<RecipeArg>,
    pub operations: Vec<ComponentOperationSchema>,
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
    #[serde(skip_serializing_if = "Option::is_none", rename = "mapsLoaded")]
    pub maps_loaded: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "mapIds")]
    pub map_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "varIds")]
    pub var_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "typeIds")]
    pub type_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<TypeSchema>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "languageIds")]
    pub language_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "queryIds")]
    pub query_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queries: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<ComponentSchema>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<StackSchema>,
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
pub struct FilePreview {
    pub path: String,
    pub kind: &'static str,
    pub is_new: bool,
    pub skipped: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub original: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<Vec<codemod_recipe_lua::TranscriptEvent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_may_diverge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<RecipeDiagnostic>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<Vec<codemod_recipe_lua::TranscriptEvent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<RecipeDiagnostic>>,
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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DumpAstResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_error: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DebugQueryResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GenerateQueryResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture_suggestion: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ResolveStaticPathResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub static_resolvable: bool,
}

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
            apply_may_diverge: None,
            diagnostics: None,
        };
        let value = serde_json::to_value(&resp).unwrap();
        assert_eq!(value["previewToken"], "abc");
    }
}
