//! Resolve and merge RawEdge lists against the flat forest.

use std::collections::{BTreeMap, BTreeSet};

use crate::component::relations::types::{ComponentEdge, FlatInstance};
use crate::component::types::ComponentFinding;

use super::findings::{relation_finding, relation_info_finding};
use super::normalize::RawEdge;
use super::targets::{member_parent_key, resolve_architecture_target};

/// Resolve raw edges against the flat forest, merging duplicates by
/// `(from, kind, target)` into one canonical edge (provenance accumulates in
/// `sources`; extras stay first-wins), and emit `relation.unresolved` findings for
/// dangling refs. Identical-provenance duplicates yield one `relation.duplicate`
/// info finding per canonical edge; distinct provenance merges silently.
///
/// When `kind` is in `member_kinds`, targets are coerced to opaque string keys
/// (no unresolved finding for free-string labels).
pub fn resolve_edges(
    raw_edges: Vec<RawEdge>,
    flat: &[FlatInstance],
    member_kinds: &std::collections::BTreeSet<String>,
) -> (Vec<ComponentEdge>, Vec<ComponentFinding>) {
    let by_id: BTreeMap<&str, &FlatInstance> = flat.iter().map(|f| (f.id.as_str(), f)).collect();
    let mut by_component: BTreeMap<&str, Vec<&FlatInstance>> = BTreeMap::new();
    for f in flat {
        by_component
            .entry(f.component_id.as_str())
            .or_default()
            .push(f);
    }

    let mut edges: Vec<ComponentEdge> = Vec::new();
    let mut findings = Vec::new();
    // Canonical edge per `(from, kind, target)` → index into `edges`.
    let mut canonical: BTreeMap<(String, String, String), usize> = BTreeMap::new();
    // All provenance values seen per canonical edge (including absent sources), so
    // identical-provenance duplicates can be told apart from distinct derivations.
    let mut provenance_seen: BTreeMap<(String, String, String), Vec<Option<serde_json::Value>>> =
        BTreeMap::new();
    let mut reported_duplicates: BTreeSet<(String, String, String)> = BTreeSet::new();
    // Findings dedupe by raw target descriptor: an unresolved duplicate carries no
    // information either (edges are facts, not events — and neither are findings).
    let mut seen_findings: BTreeSet<(String, String, String)> = BTreeSet::new();

    for raw in raw_edges {
        let is_member = member_kinds.contains(&raw.kind);
        let raw_target_key = match &raw.to {
            Some(serde_json::Value::String(s)) => format!("id:{s}"),
            Some(serde_json::Value::Object(map)) => format!(
                "ref:{}/{}",
                map.get("componentId")
                    .and_then(|v| v.as_str())
                    .unwrap_or(""),
                map.get("key").and_then(|v| v.as_str()).unwrap_or("")
            ),
            _ => raw
                .symbol
                .clone()
                .map(|s| format!("symbol:{s}"))
                .unwrap_or_default(),
        };
        if !by_id.contains_key(raw.from.as_str()) {
            if seen_findings.insert((raw.from.clone(), raw.kind.clone(), raw_target_key)) {
                findings.push(relation_finding(
                    "relation.unresolved",
                    &format!("edge '{}' has unknown from instance {}", raw.kind, raw.from),
                    &raw.from,
                    raw.source.clone(),
                ));
            }
            continue;
        }
        let (to, symbol) = if is_member {
            match member_parent_key(&raw, &by_id, &by_component) {
                Some(key) => (Some(key), None),
                None => {
                    if seen_findings.insert((raw.from.clone(), raw.kind.clone(), raw_target_key)) {
                        findings.push(relation_finding(
                            "relation.invalid",
                            &format!("member edge '{}' is missing a target (to or symbol)", raw.kind),
                            &raw.from,
                            raw.source.clone(),
                        ));
                    }
                    continue;
                }
            }
        } else {
            match resolve_architecture_target(
                &raw,
                &by_id,
                &by_component,
                &mut findings,
                &mut seen_findings,
                &raw_target_key,
            ) {
                Some(pair) => pair,
                None => continue,
            }
        };

        let target_key = to
            .clone()
            .map(|t| format!("id:{t}"))
            .or_else(|| symbol.clone().map(|s| format!("symbol:{s}")))
            .unwrap_or_default();
        let key = (raw.from.clone(), raw.kind.clone(), target_key);
        if let Some(&idx) = canonical.get(&key) {
            // Merge into the canonical edge: edges are facts, not events.
            let provenance = provenance_seen.entry(key.clone()).or_default();
            if provenance.contains(&raw.source) {
                // Same derivation seen again — redundant relate output.
                if reported_duplicates.insert(key) {
                    findings.push(relation_info_finding(
                        "relation.duplicate",
                        &format!(
                            "duplicate edge '{}' from {} (identical provenance) — adjust relate",
                            raw.kind, raw.from
                        ),
                        &raw.from,
                        raw.source.clone(),
                    ));
                }
            } else {
                if let Some(source) = &raw.source {
                    edges[idx].sources.push(source.clone());
                }
                provenance.push(raw.source.clone());
            }
            continue;
        }
        canonical.insert(key.clone(), edges.len());
        provenance_seen.insert(key, vec![raw.source.clone()]);
        edges.push(ComponentEdge {
            from: raw.from,
            kind: raw.kind,
            to,
            symbol,
            sources: raw.source.clone().into_iter().collect(),
            extra: raw.extra,
        });
    }
    (edges, findings)
}

