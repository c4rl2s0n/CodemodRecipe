//! HostCommand response / schema DTOs.

use serde::Serialize;
use std::collections::BTreeMap;

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operations: Vec<ComponentOperationSchema>,
    /// Relation-kind registry from stack (`kind → { layout }`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub relations: BTreeMap<String, RelationKindSchema>,
    /// Zone catalog from `stack.zones`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub zones: Vec<ZoneSchema>,
    /// Role → metric flags from `stack.zoneRoles`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub zone_roles: BTreeMap<String, ZoneRoleSchema>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ZoneSchema {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ZoneRoleSchema {
    pub ignore_relations: bool,
    pub ignore_coverage: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RelationKindSchema {
    pub layout: String,
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
    /// Workspace-relative path written when `out` was requested.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
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

/// Catalog view of a `*.test.luau` suite (discovery for test UIs; run via
/// [`HostCommand::RunTests`]).
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct TestSuiteSchema {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Workspace-relative path to the suite module.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Sorted test function names.
    pub test_names: Vec<String>,
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
    #[serde(skip_serializing_if = "Option::is_none", rename = "testSuites")]
    pub test_suites: Option<Vec<TestSuiteSchema>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DescribeResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recipe: Option<RecipeSchema>,
}

/// File preview DTO — owned by `codemod_recipe_components` (D-025 F extract).
pub use codemod_recipe_components::FilePreview;

/// Extract `c:log` detail strings from a transcript for MCP-facing `logs` fields.
pub fn logs_from_transcript(
    transcript: Option<&[codemod_recipe_lua::TranscriptEvent]>,
) -> Option<Vec<String>> {
    let events = transcript?;
    let logs: Vec<String> = events
        .iter()
        .filter(|e| e.kind == "log")
        .map(|e| e.detail.clone())
        .collect();
    if logs.is_empty() {
        None
    } else {
        Some(logs)
    }
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
    /// `c:log` details — top-level for MCP agents (also present as transcript kind=log).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub apply_may_diverge: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<RecipeDiagnostic>>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
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
    /// `c:log` details — top-level for MCP agents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<String>>,
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

