//! Post-scan component span coverage: union explicit leaf `source.span` ranges
//! vs stack `codebase` universe; whitespace-only exclusion; no extra tree-sitter.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Serialize;

use crate::component::types::ComponentInstance;
use crate::recipe::registry::RecipeRegistry;
use crate::sandbox::workspace_glob::glob_workspace;

const CODEBASE_REQUIRED_WARNING: &str = "stack.codebase is required to enable component coverage; add codebase = { \"…\" } globs to .codemod/stack.luau";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageTotals {
    pub ratio: f64,
    pub covered_nonblank: u64,
    pub gap_nonblank: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CoverageGap {
    pub from: usize,
    pub to: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileCoverage {
    pub path: String,
    pub ratio: f64,
    pub covered_nonblank: u64,
    pub gap_nonblank: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub gaps: Vec<CoverageGap>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentCoverageResponse {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// True when coverage was not computed (e.g. missing `codebase`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skipped: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregate: Option<CoverageTotals>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub files: Option<Vec<FileCoverage>>,
}

impl ComponentCoverageResponse {
    fn skipped_warning(warning: impl Into<String>) -> Self {
        Self {
            ok: true,
            error: None,
            skipped: Some(true),
            warning: Some(warning.into()),
            aggregate: None,
            files: None,
        }
    }

    fn fail(error: impl Into<String>) -> Self {
        Self {
            ok: false,
            error: Some(error.into()),
            skipped: None,
            warning: None,
            aggregate: None,
            files: None,
        }
    }
}

/// Byte interval `[from, to)` collected from instance trees.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpanInterval {
    pub from: usize,
    pub to: usize,
}

/// Run coverage against the current scan cache and stack `codebase`.
///
/// If no scan cache exists, runs a full component scan first.
pub fn component_coverage(registry: &mut RecipeRegistry) -> ComponentCoverageResponse {
    if registry.component_scan_cache().is_none() {
        let scan = crate::component::scan::component_scan(registry);
        if !scan.ok {
            return ComponentCoverageResponse::fail(
                scan.error
                    .unwrap_or_else(|| "component scan failed".to_string()),
            );
        }
    }

    let Some(stack) = registry.stack() else {
        return ComponentCoverageResponse::fail(
            "stack manifest is required for component coverage",
        );
    };

    let codebase = stack.codebase.clone();
    if codebase.is_empty() {
        return ComponentCoverageResponse::skipped_warning(CODEBASE_REQUIRED_WARNING);
    }

    let cache = match registry.component_scan_cache() {
        Some(c) => c,
        None => {
            return ComponentCoverageResponse::fail("component scan cache missing after scan");
        }
    };

    let spans_by_path = collect_spans_from_instances(&cache.instances);
    match compute_coverage(registry.workspace_root(), &codebase, &spans_by_path) {
        Ok((aggregate, files)) => ComponentCoverageResponse {
            ok: true,
            error: None,
            skipped: None,
            warning: None,
            aggregate: Some(aggregate),
            files: Some(files),
        },
        Err(e) => ComponentCoverageResponse::fail(e),
    }
}

pub fn collect_spans_from_instances(
    instances: &[ComponentInstance],
) -> BTreeMap<String, Vec<SpanInterval>> {
    let mut out: BTreeMap<String, Vec<SpanInterval>> = BTreeMap::new();
    for instance in instances {
        collect_spans_from_value(&instance.ast, &mut out);
    }
    out
}

/// Collect coverage-leaf spans only: a node with `source.span` contributes iff no
/// descendant also has an explicit `source.span`. Returns whether this subtree
/// contains any spanned node (leaf or not).
fn collect_spans_from_value(
    value: &serde_json::Value,
    out: &mut BTreeMap<String, Vec<SpanInterval>>,
) -> bool {
    match value {
        serde_json::Value::Object(map) => {
            let mut descendant_has_span = false;
            if let Some(children) = map.get("children").and_then(|v| v.as_array()) {
                for child in children {
                    if collect_spans_from_value(child, out) {
                        descendant_has_span = true;
                    }
                }
            }
            if let Some(fields) = map.get("fields").and_then(|v| v.as_array()) {
                for field in fields {
                    if collect_spans_from_value(field, out) {
                        descendant_has_span = true;
                    }
                }
            }

            if let Some(interval) = span_from_source(map.get("source")) {
                if !descendant_has_span {
                    if let Some(path) = map
                        .get("source")
                        .and_then(|s| s.get("path"))
                        .and_then(|v| v.as_str())
                    {
                        out.entry(path.to_string()).or_default().push(interval);
                    }
                }
                return true;
            }
            descendant_has_span
        }
        serde_json::Value::Array(items) => {
            let mut any = false;
            for item in items {
                if collect_spans_from_value(item, out) {
                    any = true;
                }
            }
            any
        }
        _ => false,
    }
}

fn span_from_source(source: Option<&serde_json::Value>) -> Option<SpanInterval> {
    let span = source?.get("span")?;
    let from = span.get("from").and_then(|v| v.as_u64())? as usize;
    let to = span.get("to").and_then(|v| v.as_u64())? as usize;
    if to < from {
        return None;
    }
    Some(SpanInterval { from, to })
}

pub fn merge_intervals(mut intervals: Vec<SpanInterval>) -> Vec<SpanInterval> {
    if intervals.is_empty() {
        return intervals;
    }
    intervals.sort_by(|a, b| a.from.cmp(&b.from).then(a.to.cmp(&b.to)));
    let mut merged = Vec::with_capacity(intervals.len());
    let mut cur = intervals[0];
    for next in intervals.into_iter().skip(1) {
        if next.from <= cur.to {
            cur.to = cur.to.max(next.to);
        } else {
            merged.push(cur);
            cur = next;
        }
    }
    merged.push(cur);
    merged
}

pub fn compute_file_coverage(source: &str, intervals: &[SpanInterval]) -> FileCoverageStats {
    let merged = merge_intervals(intervals.to_vec());
    let bytes = source.as_bytes();
    let len = bytes.len();

    let mut covered_nonblank = 0u64;
    let mut gap_nonblank = 0u64;
    let mut gaps = Vec::new();

    // Within a contiguous uncovered run: first/last nonblank byte indices.
    let mut first_nonblank: Option<usize> = None;
    let mut last_nonblank: Option<usize> = None;

    let close_uncovered_run =
        |gaps: &mut Vec<CoverageGap>, first: &mut Option<usize>, last: &mut Option<usize>| {
            if let (Some(from), Some(last_i)) = (first.take(), last.take()) {
                gaps.push(CoverageGap {
                    from,
                    to: last_i + 1,
                });
            }
        };

    let mut covered_idx = 0usize;
    let mut i = 0usize;
    while i < len {
        while covered_idx < merged.len() && merged[covered_idx].to <= i {
            covered_idx += 1;
        }
        let in_covered = covered_idx < merged.len()
            && merged[covered_idx].from <= i
            && i < merged[covered_idx].to;
        let is_ws = bytes[i].is_ascii_whitespace();

        if in_covered {
            close_uncovered_run(&mut gaps, &mut first_nonblank, &mut last_nonblank);
            if !is_ws {
                covered_nonblank += 1;
            }
        } else if is_ws {
            // Internal whitespace stays inside an open nonblank gap; do not close.
        } else {
            gap_nonblank += 1;
            if first_nonblank.is_none() {
                first_nonblank = Some(i);
            }
            last_nonblank = Some(i);
        }
        i += 1;
    }
    close_uncovered_run(&mut gaps, &mut first_nonblank, &mut last_nonblank);

    FileCoverageStats {
        covered_nonblank,
        gap_nonblank,
        gaps,
    }
}

#[derive(Debug, Clone)]
pub struct FileCoverageStats {
    pub covered_nonblank: u64,
    pub gap_nonblank: u64,
    pub gaps: Vec<CoverageGap>,
}

impl FileCoverageStats {
    pub fn ratio(&self) -> f64 {
        let total = self.covered_nonblank + self.gap_nonblank;
        if total == 0 {
            1.0
        } else {
            self.covered_nonblank as f64 / total as f64
        }
    }
}

fn compute_coverage(
    workspace_root: &Path,
    codebase: &[String],
    spans_by_path: &BTreeMap<String, Vec<SpanInterval>>,
) -> Result<(CoverageTotals, Vec<FileCoverage>), String> {
    let mut universe: Vec<String> = Vec::new();
    for pattern in codebase {
        let hits = glob_workspace(workspace_root, pattern)?;
        universe.extend(hits);
    }
    universe.sort();
    universe.dedup();

    let mut files = Vec::with_capacity(universe.len());
    let mut agg_covered = 0u64;
    let mut agg_gap = 0u64;

    for path in universe {
        let absolute = workspace_root.join(&path);
        let source = std::fs::read_to_string(&absolute)
            .map_err(|e| format!("failed to read {path}: {e}"))?;
        let empty: Vec<SpanInterval> = Vec::new();
        let intervals = spans_by_path.get(&path).unwrap_or(&empty);
        let stats = compute_file_coverage(&source, intervals);
        agg_covered += stats.covered_nonblank;
        agg_gap += stats.gap_nonblank;
        let ratio = stats.ratio();

        files.push(FileCoverage {
            path,
            ratio,
            covered_nonblank: stats.covered_nonblank,
            gap_nonblank: stats.gap_nonblank,
            gaps: stats.gaps,
        });
    }

    let aggregate = CoverageTotals {
        ratio: if agg_covered + agg_gap == 0 {
            1.0
        } else {
            agg_covered as f64 / (agg_covered + agg_gap) as f64
        },
        covered_nonblank: agg_covered,
        gap_nonblank: agg_gap,
    };
    Ok((aggregate, files))
}

/// Compact coverage for scan snapshots: aggregate + per-file ratios, **no** gaps.
/// Returns `None` when `codebase` is empty.
///
/// `exclude_instance_ids`: root instances whose leaf spans are omitted (zone
/// `ignoreCoverage` members). Live `component_coverage` does not use this filter.
pub fn compact_coverage_for_snapshot(
    workspace_root: &Path,
    codebase: &[String],
    instances: &[ComponentInstance],
    exclude_instance_ids: &std::collections::BTreeSet<String>,
) -> Result<Option<serde_json::Value>, String> {
    if codebase.is_empty() {
        return Ok(None);
    }
    let filtered: Vec<&ComponentInstance> = instances
        .iter()
        .filter(|i| !exclude_instance_ids.contains(&i.id))
        .collect();
    let owned: Vec<ComponentInstance> = filtered.into_iter().cloned().collect();
    let spans = collect_spans_from_instances(&owned);
    let (aggregate, files) = compute_coverage(workspace_root, codebase, &spans)?;
    let files_compact: Vec<serde_json::Value> = files
        .into_iter()
        .map(|f| {
            serde_json::json!({
                "path": f.path,
                "ratio": f.ratio,
                "coveredNonblank": f.covered_nonblank,
                "gapNonblank": f.gap_nonblank,
            })
        })
        .collect();
    Ok(Some(serde_json::json!({
        "aggregate": aggregate,
        "files": files_compact,
    })))
}

/// Test helper: coverage from an existing cache without re-scan.
#[cfg(test)]
pub fn coverage_from_cache(
    workspace_root: &Path,
    codebase: &[String],
    cache: &crate::component::cache::ComponentScanCache,
) -> Result<(CoverageTotals, Vec<FileCoverage>), String> {
    let spans = collect_spans_from_instances(&cache.instances);
    compute_coverage(workspace_root, codebase, &spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_overlaps_and_adjacent() {
        let merged = merge_intervals(vec![
            SpanInterval { from: 0, to: 5 },
            SpanInterval { from: 3, to: 8 },
            SpanInterval { from: 10, to: 12 },
            SpanInterval { from: 12, to: 15 },
        ]);
        assert_eq!(
            merged,
            vec![
                SpanInterval { from: 0, to: 8 },
                SpanInterval { from: 10, to: 15 },
            ]
        );
    }

    #[test]
    fn sweep_whitespace_excluded_comments_count_as_gaps() {
        // "a\n//c\nb" — cover only 'a' and 'b'
        let source = "a\n//c\nb";
        // a at 0, \n at 1, //c at 2..5, \n at 5, b at 6
        let stats = compute_file_coverage(
            source,
            &[
                SpanInterval { from: 0, to: 1 },
                SpanInterval { from: 6, to: 7 },
            ],
        );
        assert_eq!(stats.covered_nonblank, 2);
        assert_eq!(stats.gap_nonblank, 3);
        assert_eq!(stats.gaps.len(), 1);
        assert_eq!(stats.gaps[0].from, 2);
        assert_eq!(stats.gaps[0].to, 5);
        assert!((stats.ratio() - 2.0 / 5.0).abs() < 1e-9);
    }

    #[test]
    fn consolidated_gap_includes_internal_whitespace() {
        // Cover ends, leave "x\n\ny" uncovered as one gap.
        let source = "A x\n\ny B";
        // A=0, space=1, x=2, \n=3, \n=4, y=5, space=6, B=7
        let stats = compute_file_coverage(
            source,
            &[
                SpanInterval { from: 0, to: 1 },
                SpanInterval { from: 7, to: 8 },
            ],
        );
        assert_eq!(stats.covered_nonblank, 2);
        assert_eq!(stats.gap_nonblank, 2); // x and y
        assert_eq!(stats.gaps.len(), 1);
        assert_eq!(stats.gaps[0].from, 2);
        assert_eq!(stats.gaps[0].to, 6); // past last nonblank 'y'
    }

    #[test]
    fn whitespace_only_uncovered_run_emits_no_gap() {
        let source = "a   b";
        let stats = compute_file_coverage(
            source,
            &[
                SpanInterval { from: 0, to: 1 },
                SpanInterval { from: 4, to: 5 },
            ],
        );
        assert_eq!(stats.covered_nonblank, 2);
        assert_eq!(stats.gap_nonblank, 0);
        assert!(stats.gaps.is_empty());
    }

    #[test]
    fn all_whitespace_file_is_full_coverage() {
        let stats = compute_file_coverage("  \n\t", &[]);
        assert_eq!(stats.covered_nonblank, 0);
        assert_eq!(stats.gap_nonblank, 0);
        assert_eq!(stats.ratio(), 1.0);
        assert!(stats.gaps.is_empty());
    }

    #[test]
    fn path_only_source_does_not_collect_span() {
        let node = serde_json::json!({
            "id": "x",
            "source": { "path": "lib/a.dart" },
            "children": [{
                "source": { "path": "lib/a.dart", "span": { "from": 1, "to": 4 } }
            }]
        });
        let mut out = BTreeMap::new();
        collect_spans_from_value(&node, &mut out);
        assert_eq!(out["lib/a.dart"].len(), 1);
        assert_eq!(out["lib/a.dart"][0], SpanInterval { from: 1, to: 4 });
    }

    #[test]
    fn parent_span_ignored_when_children_have_spans() {
        let node = serde_json::json!({
            "source": { "path": "lib/a.dart", "span": { "from": 0, "to": 100 } },
            "children": [
                { "source": { "path": "lib/a.dart", "span": { "from": 10, "to": 20 } } },
                { "source": { "path": "lib/a.dart", "span": { "from": 50, "to": 60 } } }
            ]
        });
        let mut out = BTreeMap::new();
        assert!(collect_spans_from_value(&node, &mut out));
        let spans = &out["lib/a.dart"];
        assert_eq!(spans.len(), 2);
        assert!(spans.contains(&SpanInterval { from: 10, to: 20 }));
        assert!(spans.contains(&SpanInterval { from: 50, to: 60 }));
        assert!(!spans.iter().any(|s| s.from == 0 && s.to == 100));
    }

    #[test]
    fn spanned_parent_with_path_only_children_counts_as_leaf() {
        let node = serde_json::json!({
            "source": { "path": "lib/a.dart", "span": { "from": 5, "to": 15 } },
            "children": [
                { "source": { "path": "lib/a.dart" } }
            ]
        });
        let mut out = BTreeMap::new();
        collect_spans_from_value(&node, &mut out);
        assert_eq!(out["lib/a.dart"], vec![SpanInterval { from: 5, to: 15 }]);
    }

    #[test]
    fn compact_snapshot_excludes_ignored_instance_spans() {
        let root = std::env::temp_dir().join(format!(
            "codemod_cov_exclude_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let src = "abcdefghij"; // 10 nonblank
        std::fs::write(root.join("a.rs"), src).unwrap();
        let keep = ComponentInstance {
            id: "keep/a".to_string(),
            component_id: "keep".to_string(),
            locator: serde_json::json!({}),
            ast: serde_json::json!({
                "id": "a",
                "source": { "path": "a.rs", "span": { "from": 0, "to": 5 } }
            }),
            summary: None,
        };
        let drop = ComponentInstance {
            id: "drop/b".to_string(),
            component_id: "drop".to_string(),
            locator: serde_json::json!({}),
            ast: serde_json::json!({
                "id": "b",
                "source": { "path": "a.rs", "span": { "from": 5, "to": 10 } }
            }),
            summary: None,
        };
        let all = vec![keep, drop];
        let none: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        let full = compact_coverage_for_snapshot(&root, &["a.rs".into()], &all, &none)
            .unwrap()
            .unwrap();
        assert_eq!(full["aggregate"]["coveredNonblank"], 10);

        let mut exclude = std::collections::BTreeSet::new();
        exclude.insert("drop/b".to_string());
        let filtered = compact_coverage_for_snapshot(&root, &["a.rs".into()], &all, &exclude)
            .unwrap()
            .unwrap();
        assert_eq!(filtered["aggregate"]["coveredNonblank"], 5);
        assert_eq!(filtered["aggregate"]["gapNonblank"], 5);
        let _ = std::fs::remove_dir_all(&root);
    }
}
