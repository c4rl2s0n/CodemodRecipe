use std::collections::BTreeMap;
use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::input_kind::InputKind;

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct Recipe {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub args: Vec<Arg>,
    pub maps: BTreeMap<String, BTreeMap<String, String>>,
    pub types: BTreeMap<String, TypeDef>,
    pub queries: BTreeMap<String, String>,
    pub post_execution: Vec<String>,
    pub explorer_menu: Vec<ExplorerMenuEntry>,
    #[schemars(skip)]
    pub source: String,
    #[schemars(skip)]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct TypeDef {
    pub id: String,
    pub description: Option<String>,
    pub fields: Vec<Arg>,
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct Arg {
    pub name: String,
    pub required: bool,
    pub input_kind: InputKind,
    pub abbr: Option<String>,
    pub help: Option<String>,
    pub defaults_to: Option<String>,
    pub options: Vec<String>,
    pub options_from_map: Option<String>,
    pub allow_custom_value: Option<bool>,
    pub from: Option<ArgFrom>,
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub enum ArgFrom {
    Builtin(String),
    Spec(ArgFromSpec),
}

#[derive(Debug, Clone, PartialEq, Eq, Default, JsonSchema)]
pub struct ArgFromSpec {
    pub template: Option<String>,
    pub query: Option<String>,
    pub capture: Option<String>,
    pub extract: Option<String>,
    pub scope: Option<String>,
    pub language: Option<String>,
    pub on_no_match: Option<String>,
    pub on_many_matches: Option<String>,
    pub join: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, JsonSchema)]
pub enum ExplorerMenuKind {
    File,
    Folder,
}

impl ExplorerMenuKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Folder => "folder",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "file" => Some(Self::File),
            "folder" => Some(Self::Folder),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct ExplorerMenuEntry {
    pub kind: ExplorerMenuKind,
    pub if_expr: Option<String>,
    pub args: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct MapAsset {
    pub id: String,
    pub description: Option<String>,
    pub map: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct VariablesAsset {
    pub id: String,
    pub description: Option<String>,
    pub values: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct QueryLibAsset {
    pub id: String,
    pub queries: BTreeMap<String, String>,
}

/// Root-scan stack manifest (`.codemod/stack.luau`).
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct StackDef {
    pub id: String,
    pub description: Option<String>,
    /// Component ids to run at workspace root scan.
    pub components: Vec<String>,
    #[schemars(skip)]
    pub source: String,
    #[schemars(skip)]
    pub path: Option<PathBuf>,
}

/// Component kind module (`identify` / `parse` / `verify` hooks).
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct ComponentDef {
    pub id: String,
    pub description: Option<String>,
    pub operations: Vec<OperationDef>,
    /// True when the module defines a top-level `identify` function.
    pub has_identify: bool,
    /// True when the module defines a top-level `toSummary` function.
    pub has_to_summary: bool,
    /// True when the module defines a top-level `scaffold` table with `apply`.
    pub has_scaffold: bool,
    /// Args declared on `scaffold.args` (empty when no scaffold or no args table).
    pub scaffold_args: Vec<Arg>,
    #[schemars(skip)]
    pub source: String,
    #[schemars(skip)]
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
pub struct OperationDef {
    pub name: String,
    pub description: Option<String>,
    pub args: Vec<Arg>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptStats {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub added_lines: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removed_lines: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptEvent {
    pub kind: String,
    pub detail: String,
    #[serde(default)]
    pub included_in_preview: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats: Option<TranscriptStats>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub expandable: bool,
}

impl TranscriptEvent {
    pub fn simple(kind: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            detail: detail.into(),
            included_in_preview: true,
            stats: None,
            payload: None,
            expandable: false,
        }
    }

    pub fn with_payload(
        kind: impl Into<String>,
        detail: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        let expandable = payload_is_expandable(&payload);
        Self {
            kind: kind.into(),
            detail: detail.into(),
            included_in_preview: true,
            stats: None,
            payload: Some(payload),
            expandable,
        }
    }

    pub fn edit(
        kind: impl Into<String>,
        detail: impl Into<String>,
        stats: TranscriptStats,
        payload: serde_json::Value,
    ) -> Self {
        let expandable = payload_is_expandable(&payload);
        let stats = normalize_stats(stats);
        Self {
            kind: kind.into(),
            detail: detail.into(),
            included_in_preview: true,
            stats,
            payload: Some(payload),
            expandable,
        }
    }
}

fn payload_is_expandable(payload: &serde_json::Value) -> bool {
    match payload {
        serde_json::Value::Null => false,
        serde_json::Value::Object(map) => !map.is_empty(),
        serde_json::Value::Array(arr) => !arr.is_empty(),
        serde_json::Value::String(s) => !s.is_empty(),
        _ => true,
    }
}

fn normalize_stats(stats: TranscriptStats) -> Option<TranscriptStats> {
    let added = stats.added_lines.filter(|&n| n > 0);
    let removed = stats.removed_lines.filter(|&n| n > 0);
    if added.is_none() && removed.is_none() {
        None
    } else {
        Some(TranscriptStats {
            added_lines: added,
            removed_lines: removed,
        })
    }
}
