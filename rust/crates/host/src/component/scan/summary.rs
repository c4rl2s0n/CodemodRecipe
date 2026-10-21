//! Compact scan summary for MCP / agent responses.

use std::collections::BTreeMap;

use crate::component::relation_metrics::{ignore_coverage_instance_ids, relation_metrics};
use crate::component::scan::types::ScanData;
use crate::component::snapshot::DEFAULT_SNAPSHOT_REL;
use crate::component::types::{ComponentScanSummary, ScanGapEntry};
use crate::recipe::registry::RecipeRegistry;

const TOP_GAPS: usize = 8;

fn is_product_path(path: &str) -> bool {
    if path.contains("/tests/") || path.contains("\\tests\\") {
        return false;
    }
    if path.contains(".test.") || path.ends_with("_test.rs") || path.ends_with("_test.ts") {
        return false;
    }
    if path.ends_with(".spec.ts") {
        return false;
    }
    true
}

pub(super) fn build_scan_summary(
    registry: &RecipeRegistry,
    data: &ScanData,
) -> ComponentScanSummary {
    let findings = data.all_findings();
    let mut findings_by_severity: BTreeMap<String, usize> = BTreeMap::new();
    let mut dead_composers = Vec::new();
    for f in &findings {
        *findings_by_severity
            .entry(f.severity.clone())
            .or_insert(0) += 1;
        if f.code == "composer.identify_empty" {
            if let Some(id) = f.message.split('\'').nth(1) {
                dead_composers.push(id.to_string());
            }
        }
    }
    dead_composers.sort();
    dead_composers.dedup();

    let relations_reg = registry
        .stack()
        .map(|s| s.relations.clone())
        .unwrap_or_default();
    let zones = registry
        .stack()
        .map(|s| s.zones.clone())
        .unwrap_or_default();
    let zone_roles = registry
        .stack()
        .map(|s| s.zone_roles.clone())
        .unwrap_or_default();

    let metrics = relation_metrics(
        &data.instances,
        &data.edges,
        &relations_reg,
        &zones,
        &zone_roles,
    );

    let mut span_ratio = None;
    let mut product_span_ratio = None;
    let mut top_product_gaps = Vec::new();
    let codebase = registry
        .stack()
        .map(|s| s.codebase.clone())
        .unwrap_or_default();
    if !codebase.is_empty() {
        let exclude = ignore_coverage_instance_ids(
            &data.instances,
            &data.edges,
            &relations_reg,
            &zones,
            &zone_roles,
        );
        if let Ok(Some(cov)) = crate::component::coverage::compact_coverage_for_snapshot(
            registry.workspace_root(),
            &codebase,
            &data.instances,
            &exclude,
        ) {
            if let Some(agg) = cov.get("aggregate") {
                span_ratio = agg.get("ratio").and_then(|v| v.as_f64());
            }
            if let Some(files) = cov.get("files").and_then(|v| v.as_array()) {
                let mut product_covered: u64 = 0;
                let mut product_gap: u64 = 0;
                let mut gaps: Vec<ScanGapEntry> = files
                    .iter()
                    .filter_map(|f| {
                        let path = f.get("path")?.as_str()?;
                        if !is_product_path(path) {
                            return None;
                        }
                        let covered = f.get("coveredNonblank")?.as_u64().unwrap_or(0);
                        let gap = f.get("gapNonblank")?.as_u64().unwrap_or(0);
                        product_covered += covered;
                        product_gap += gap;
                        Some(ScanGapEntry {
                            path: path.to_string(),
                            gap_nonblank: gap,
                            ratio: f.get("ratio")?.as_f64().unwrap_or(0.0),
                        })
                    })
                    .filter(|g| g.gap_nonblank > 0)
                    .collect();
                let product_total = product_covered + product_gap;
                if product_total > 0 {
                    product_span_ratio = Some(product_covered as f64 / product_total as f64);
                }
                gaps.sort_by(|a, b| b.gap_nonblank.cmp(&a.gap_nonblank));
                gaps.truncate(TOP_GAPS);
                top_product_gaps = gaps;
            }
        }
    }

    ComponentScanSummary {
        architecture_connected: metrics.aggregate.architecture_connected,
        architecture_eligible: metrics.aggregate.architecture_eligible,
        architecture_isolated: metrics.aggregate.architecture_isolated,
        instance_count: data.instances.len(),
        edge_count: data.edges.len(),
        finding_count: findings.len(),
        findings_by_severity,
        dead_composers,
        span_ratio,
        product_span_ratio,
        top_product_gaps,
        snapshot_path: Some(DEFAULT_SNAPSHOT_REL.to_string()),
    }
}
