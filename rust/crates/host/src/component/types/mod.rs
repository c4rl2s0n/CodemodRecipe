//! Component scan / invoke response shapes and parse helpers.

mod parse;

use serde::Serialize;

use crate::protocol::RecipeDiagnostic;

pub use parse::{
    context_path, expand_parse_results, instance_id, normalize_findings, normalize_parse_context,
    soft_validate_parsed,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInstance {
    pub id: String,
    pub component_id: String,
    pub locator: serde_json::Value,
    /// Parse-hook payload (the AST projection). Named `ast` so "node" only ever
    /// means a graph/tree vertex.
    pub ast: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentFinding {
    pub severity: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locator: Option<serde_json::Value>,
}

/// Compact agent-facing scan roll-up (full instances/edges still on the response).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentScanSummary {
    pub architecture_connected: usize,
    pub architecture_eligible: usize,
    pub architecture_isolated: usize,
    pub instance_count: usize,
    pub edge_count: usize,
    pub finding_count: usize,
    pub findings_by_severity: std::collections::BTreeMap<String, usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dead_composers: Vec<String>,
    /// Full codebase span ratio (includes tests when they are in codebase globs).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub span_ratio: Option<f64>,
    /// Span ratio over product paths only (excludes `**/tests/**`, `*.test.*`, etc.).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product_span_ratio: Option<f64>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub top_product_gaps: Vec<ScanGapEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanGapEntry {
    pub path: String,
    pub gap_nonblank: u64,
    pub ratio: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentScanResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ComponentScanSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instances: Option<Vec<ComponentInstance>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub findings: Option<Vec<ComponentFinding>>,
    /// Resolved relation edges over the scanned forest (relate phase).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edges: Option<Vec<crate::component::relations::ComponentEdge>>,
    /// Kind-level aggregation of edges: `(fromComponentId, kind, toComponentId)` + count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind_graph: Option<Vec<crate::component::relations::KindEdge>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostics: Option<Vec<RecipeDiagnostic>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentInvokeResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// JSON return value of read hooks (`identify` / `parse` / `verify` / `toSummary`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<ComponentInstance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub findings: Option<Vec<ComponentFinding>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<crate::protocol::FilePreview>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcript: Option<Vec<codemod_recipe_lua::TranscriptEvent>>,
    /// `c:log` details — top-level for MCP agents (also in transcript as kind=log).
    /// Absent on `component_scan` (scan has no transcript).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied: Option<Vec<String>>,
}

impl ComponentInvokeResponse {
    pub fn fail(error: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: Some(error.into()),
            result: None,
            instance: None,
            findings: None,
            files: None,
            transcript: None,
            logs: None,
            applied: None,
        }
    }

    /// Attach transcript and derive top-level `logs` from `c:log` events.
    pub fn with_transcript(
        mut self,
        transcript: Option<Vec<codemod_recipe_lua::TranscriptEvent>>,
    ) -> Self {
        self.logs = crate::protocol::logs_from_transcript(transcript.as_deref());
        self.transcript = transcript;
        self
    }
}
