//! Scan-cache data shapes.

use std::collections::BTreeSet;


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileFingerprint {
    pub mtime_secs: u64,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RootKey {
    pub component_id: String,
    pub root_path: String,
}

#[derive(Debug, Clone)]
pub struct NewRoot {
    pub component_id: String,
    pub locator: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct RootEntry {
    pub instance_id: String,
    pub component_id: String,
    pub root_path: String,
    pub locator: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct DetectResult {
    pub dirty_paths: BTreeSet<String>,
    pub new_roots: Vec<NewRoot>,
    pub removed_roots: Vec<RootKey>,
    pub roots_to_reparse: BTreeSet<String>,
}
