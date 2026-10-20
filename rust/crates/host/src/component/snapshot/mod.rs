//! Persist deterministic component scan snapshots under the codemod root.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod io;
use io::{resolve_codemod_relative, should_write_json, unix_timestamp_secs, write_text_if_changed};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;

use crate::component::graph_render::{render_instance_graph_mermaid, render_kind_graph_mermaid};
use crate::component::relation_metrics::{ignore_coverage_instance_ids, relation_metrics};
use crate::component::relations::flatten_instances;
use crate::component::types::ComponentScanResponse;
use crate::recipe::registry::RecipeRegistry;
use crate::sandbox::path_sandbox::PathSandbox;

/// On-disk snapshot JSON with stable field order (serde struct, not BTreeMap).
#[derive(Serialize)]
struct ScanSnapshotFile<'a> {
    meta: SnapshotMeta<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    coverage: Option<&'a serde_json::Value>,
    relations: &'a serde_json::Value,
    instances: &'a serde_json::Value,
    edges: &'a serde_json::Value,
    #[serde(rename = "kindGraph")]
    kind_graph: &'a serde_json::Value,
    findings: &'a serde_json::Value,
}

#[derive(Serialize)]
struct SnapshotMeta<'a> {
    version: u32,
    timestamp: u64,
    stack: &'a str,
}

pub const DEFAULT_SNAPSHOT_REL: &str = ".cache/scan-snapshot.json";
pub const DEFAULT_KIND_GRAPH_REL: &str = ".cache/kind-graph.mmd";
pub const DEFAULT_INSTANCE_GRAPH_REL: &str = ".cache/instance-graph.mmd";

#[derive(Debug, Clone)]
pub struct SnapshotWriteResult {
    pub path: String,
    pub wrote: bool,
    pub kind_graph_path: String,
    pub instance_graph_path: String,
    pub graphs_wrote: bool,
    pub instance_count: usize,
    pub edge_count: usize,
    pub finding_count: usize,
}

/// Build + optionally write a scan snapshot under `codemod_root`.
///
/// `relative` is codemod-root-relative; empty/`None` → [`DEFAULT_SNAPSHOT_REL`].
/// Rewrites the JSON only when non-meta payload content changed.
/// Always also refreshes kind + instance mermaid under `.cache/` (content-stable).
pub fn persist_scan_snapshot(
    registry: &RecipeRegistry,
    scan: &ComponentScanResponse,
    relative: Option<&str>,
) -> Result<SnapshotWriteResult, String> {
    if !scan.ok {
        return Err(scan
            .error
            .clone()
            .unwrap_or_else(|| "component scan failed".to_string()));
    }

    let rel = relative
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_SNAPSHOT_REL);

    let mut instances = scan.instances.clone().unwrap_or_default();
    instances.sort_by(|a, b| a.id.cmp(&b.id));
    let mut edges = scan.edges.clone().unwrap_or_default();
    edges.sort_by(|a, b| {
        (&a.from, &a.kind, a.to.as_deref().unwrap_or("")).cmp(&(
            &b.from,
            &b.kind,
            b.to.as_deref().unwrap_or(""),
        ))
    });
    let mut kind_graph = scan.kind_graph.clone().unwrap_or_default();
    kind_graph.sort_by(|a, b| (&a.from, &a.kind, &a.to).cmp(&(&b.from, &b.kind, &b.to)));
    let mut findings = scan.findings.clone().unwrap_or_default();
    findings.sort_by(|a, b| {
        (
            a.code.as_str(),
            a.instance_id.as_deref().unwrap_or(""),
            a.message.as_str(),
        )
            .cmp(&(
                b.code.as_str(),
                b.instance_id.as_deref().unwrap_or(""),
                b.message.as_str(),
            ))
    });

    let stack_id = registry.stack().map(|s| s.id.clone()).unwrap_or_default();
    let codebase = registry
        .stack()
        .map(|s| s.codebase.clone())
        .unwrap_or_default();
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

    let exclude_cov: BTreeSet<String> = ignore_coverage_instance_ids(
        &instances,
        &edges,
        &relations_reg,
        &zones,
        &zone_roles,
    );
    let coverage = crate::component::coverage::compact_coverage_for_snapshot(
        registry.workspace_root(),
        &codebase,
        &instances,
        &exclude_cov,
    )?;

    let relations = relation_metrics(
        &instances,
        &edges,
        &relations_reg,
        &zones,
        &zone_roles,
    );
    let relations_json = serde_json::to_value(&relations).map_err(|e| e.to_string())?;

    let mut content = serde_json::json!({
        "instances": instances,
        "edges": edges,
        "kindGraph": kind_graph,
        "findings": findings,
        "relations": relations_json,
    });
    if let Some(cov) = &coverage {
        content["coverage"] = cov.clone();
    }
    let instance_count = content["instances"]
        .as_array()
        .map(|a| a.len())
        .unwrap_or(0);
    let edge_count = content["edges"].as_array().map(|a| a.len()).unwrap_or(0);
    let finding_count = content["findings"].as_array().map(|a| a.len()).unwrap_or(0);

    let abs = resolve_codemod_relative(registry.codemod_root(), rel)?;
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    let wrote = should_write_json(&abs, &content)?;
    if wrote {
        let payload = ScanSnapshotFile {
            meta: SnapshotMeta {
                version: 1,
                timestamp: unix_timestamp_secs(),
                stack: stack_id.as_str(),
            },
            coverage: content.get("coverage"),
            relations: &content["relations"],
            instances: &content["instances"],
            edges: &content["edges"],
            kind_graph: &content["kindGraph"],
            findings: &content["findings"],
        };
        let mut body = serde_json::to_vec_pretty(&payload).map_err(|e| e.to_string())?;
        body.push(b'\n');
        std::fs::write(&abs, body).map_err(|e| e.to_string())?;
    }

    let flat = flatten_instances(&instances);
    let member_kinds = crate::component::membership::member_kinds(&relations_reg);
    let kinds_mmd = render_kind_graph_mermaid(&kind_graph);
    let instances_mmd = render_instance_graph_mermaid(&edges, &flat, &member_kinds);
    let kinds_wrote =
        write_text_if_changed(registry.codemod_root(), DEFAULT_KIND_GRAPH_REL, &kinds_mmd)?;
    let instances_wrote = write_text_if_changed(
        registry.codemod_root(),
        DEFAULT_INSTANCE_GRAPH_REL,
        &instances_mmd,
    )?;

    Ok(SnapshotWriteResult {
        path: rel.to_string(),
        wrote,
        kind_graph_path: DEFAULT_KIND_GRAPH_REL.to_string(),
        instance_graph_path: DEFAULT_INSTANCE_GRAPH_REL.to_string(),
        graphs_wrote: kinds_wrote || instances_wrote,
        instance_count,
        edge_count,
        finding_count,
    })
}

