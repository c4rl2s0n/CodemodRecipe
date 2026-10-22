//! Scan request/response data shapes.

use crate::component::relations::{ComponentEdge, KindEdge};
use crate::component::types::{ComponentFinding, ComponentInstance};

#[derive(Default)]
pub struct ComponentIncrementalScanRequest {
    pub dirty_paths: Vec<String>,
    pub detect_changes: bool,
    pub force: bool,
}

/// Everything a scan produces, split by origin so incremental merges stay precise.
#[derive(Debug, Clone, Default)]
pub struct ScanData {
    pub instances: Vec<ComponentInstance>,
    /// Parse-time findings (soft validation) — merged per-instance on incremental scans.
    pub parse_findings: Vec<ComponentFinding>,
    /// Relation + verify findings — recomputed wholesale over the merged forest.
    pub judgment_findings: Vec<ComponentFinding>,
    pub edges: Vec<ComponentEdge>,
    pub kind_graph: Vec<KindEdge>,
}

impl ScanData {
    pub fn all_findings(&self) -> Vec<ComponentFinding> {
        let mut findings = self.parse_findings.clone();
        findings.extend(self.judgment_findings.clone());
        findings
    }
}
