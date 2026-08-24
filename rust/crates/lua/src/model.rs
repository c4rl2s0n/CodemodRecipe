use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipe {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub args: Vec<Arg>,
    pub maps: BTreeMap<String, BTreeMap<String, String>>,
    pub queries: BTreeMap<String, String>,
    pub post_execution: Vec<String>,
    pub explorer_menu: Vec<ExplorerMenuEntry>,
    pub source: String,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arg {
    pub name: String,
    pub required: bool,
    pub input_kind: Option<String>,
    pub abbr: Option<String>,
    pub help: Option<String>,
    pub defaults_to: Option<String>,
    pub options: Vec<String>,
    pub allow_custom_value: Option<bool>,
    pub from: Option<ArgFrom>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgFrom {
    Builtin(String),
    Spec(ArgFromSpec),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExplorerMenuEntry {
    pub kind: ExplorerMenuKind,
    pub if_expr: Option<String>,
    pub args: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapAsset {
    pub id: String,
    pub description: Option<String>,
    pub map: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariablesAsset {
    pub id: String,
    pub description: Option<String>,
    pub values: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryLibAsset {
    pub id: String,
    pub queries: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptEvent {
    pub kind: String,
    pub detail: String,
    #[serde(default)]
    pub included_in_preview: bool,
}
