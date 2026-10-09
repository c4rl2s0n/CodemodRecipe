//! Normalize and resolve relate-hook edges; kind-level roll-up.

use std::collections::{BTreeMap, BTreeSet};

use crate::component::relations::types::{ComponentEdge, FlatInstance, KindEdge};
use crate::component::types::ComponentFinding;

/// An edge table as returned from a component's `relate` hook (pre-resolution).
pub struct RawEdge {
    pub from: String,
    pub kind: String,
    pub to: Option<serde_json::Value>,
    pub symbol: Option<String>,
    pub source: Option<serde_json::Value>,
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Normalize a `relate` return value: nil → none, object → one edge, array → many.
/// `default_from` is the relating instance's flat id.
pub fn normalize_raw_edges(
    raw: serde_json::Value,
    default_from: &str,
) -> (Vec<RawEdge>, Vec<ComponentFinding>) {
    let items = match raw {
        serde_json::Value::Null => return (Vec::new(), Vec::new()),
        serde_json::Value::Array(arr) => arr,
        // Empty Luau table `{}` converts to a JSON object, not an array —
        // an empty relate result means no edges, not one empty edge.
        serde_json::Value::Object(ref map) if map.is_empty() => {
            return (Vec::new(), Vec::new());
        }
        obj @ serde_json::Value::Object(_) => vec![obj],
        _ => {
            return (
                Vec::new(),
                vec![relation_finding(
                    "relation.invalid",
                    "relate must return an edge table, an array of edges, or nil",
                    default_from,
                    None,
                )],
            );
        }
    };
    let mut edges = Vec::new();
    let mut findings = Vec::new();
    for item in items {
        let Some(obj) = item.as_object() else {
            findings.push(relation_finding(
                "relation.invalid",
                "edge must be a table",
                default_from,
                None,
            ));
            continue;
        };
        let from = obj
            .get("from")
            .and_then(|v| v.as_str())
            .unwrap_or(default_from)
            .to_string();
        let Some(kind) = obj.get("kind").and_then(|v| v.as_str()) else {
            findings.push(relation_finding(
                "relation.invalid",
                "edge is missing kind",
                &from,
                obj.get("source").cloned(),
            ));
            continue;
        };
        let source = obj.get("source").cloned();
        // Target forms: `to` as instanceId string / `{ componentId, key }` ref /
        // `{ symbol = "..." }` table, or top-level `symbol = "..."`.
        let mut to = obj.get("to").cloned();
        let mut symbol = obj
            .get("symbol")
            .and_then(|v| v.as_str())
            .map(str::to_string);
        if let Some(serde_json::Value::Object(map)) = &to {
            if let Some(sym) = map.get("symbol").and_then(|v| v.as_str()) {
                symbol = Some(sym.to_string());
                to = None;
            }
        }
        if to.is_none() && symbol.is_none() {
            findings.push(relation_finding(
                "relation.invalid",
                &format!("edge '{kind}' is missing a target (to or symbol)"),
                &from,
                source,
            ));
            continue;
        }
        let mut extra = BTreeMap::new();
        for (key, value) in obj {
            if !matches!(key.as_str(), "from" | "kind" | "to" | "symbol" | "source") {
                extra.insert(key.clone(), value.clone());
            }
        }
        edges.push(RawEdge {
            from,
            kind: kind.to_string(),
            to,
            symbol,
            source,
            extra,
        });
    }
    (edges, findings)
}

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

/// Coerce a member-kind target to an opaque subgraph key string.
fn member_parent_key(
    raw: &RawEdge,
    by_id: &BTreeMap<&str, &FlatInstance>,
    by_component: &BTreeMap<&str, Vec<&FlatInstance>>,
) -> Option<String> {
    match &raw.to {
        None => raw.symbol.clone().filter(|s| !s.is_empty()),
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        Some(serde_json::Value::Object(map)) => {
            if let Some(sym) = map.get("symbol").and_then(|v| v.as_str()) {
                return Some(sym.to_string());
            }
            let component_id = map.get("componentId").and_then(|v| v.as_str())?;
            let key = map.get("key").and_then(|v| v.as_str())?;
            let direct = format!("{component_id}/{key}");
            if by_id.contains_key(direct.as_str()) {
                return Some(direct);
            }
            if let Some(id) = by_component.get(component_id).and_then(|candidates| {
                candidates
                    .iter()
                    .find(|f| f.ast.get("id").and_then(|v| v.as_str()) == Some(key))
                    .map(|f| f.id.clone())
            }) {
                return Some(id);
            }
            // Unresolved ref still becomes a usable opaque label.
            Some(direct)
        }
        Some(_) => None,
    }
}

fn resolve_architecture_target(
    raw: &RawEdge,
    by_id: &BTreeMap<&str, &FlatInstance>,
    by_component: &BTreeMap<&str, Vec<&FlatInstance>>,
    findings: &mut Vec<ComponentFinding>,
    seen_findings: &mut BTreeSet<(String, String, String)>,
    raw_target_key: &str,
) -> Option<(Option<String>, Option<String>)> {
    match &raw.to {
        None => Some((None, raw.symbol.clone())),
        Some(serde_json::Value::String(s)) => {
            if by_id.contains_key(s.as_str()) {
                Some((Some(s.clone()), None))
            } else {
                if seen_findings.insert((raw.from.clone(), raw.kind.clone(), raw_target_key.to_string()))
                {
                    findings.push(relation_finding(
                        "relation.unresolved",
                        &format!("{} --{}--> unresolved instance id {s}", raw.from, raw.kind),
                        &raw.from,
                        raw.source.clone(),
                    ));
                }
                None
            }
        }
        Some(serde_json::Value::Object(map)) => {
            let component_id = map.get("componentId").and_then(|v| v.as_str());
            let key = map.get("key").and_then(|v| v.as_str());
            match (component_id, key) {
                (Some(component_id), Some(key)) => {
                    let direct = format!("{component_id}/{key}");
                    let resolved = if by_id.contains_key(direct.as_str()) {
                        Some(direct)
                    } else {
                        by_component
                            .get(component_id)
                            .and_then(|candidates| {
                                candidates.iter().find(|f| {
                                    f.ast.get("id").and_then(|v| v.as_str()) == Some(key)
                                })
                            })
                            .map(|f| f.id.clone())
                    };
                    match resolved {
                        Some(id) => Some((Some(id), None)),
                        None => {
                            if seen_findings.insert((
                                raw.from.clone(),
                                raw.kind.clone(),
                                raw_target_key.to_string(),
                            )) {
                                findings.push(relation_finding(
                                    "relation.unresolved",
                                    &format!(
                                        "{} --{}--> unresolved ref {component_id}/{key}",
                                        raw.from, raw.kind
                                    ),
                                    &raw.from,
                                    raw.source.clone(),
                                ));
                            }
                            None
                        }
                    }
                }
                _ => {
                    if seen_findings.insert((
                        raw.from.clone(),
                        raw.kind.clone(),
                        raw_target_key.to_string(),
                    )) {
                        findings.push(relation_finding(
                            "relation.invalid",
                            &format!(
                                "edge '{}' target table requires componentId + key",
                                raw.kind
                            ),
                            &raw.from,
                            raw.source.clone(),
                        ));
                    }
                    None
                }
            }
        }
        Some(_) => {
            if seen_findings.insert((
                raw.from.clone(),
                raw.kind.clone(),
                raw_target_key.to_string(),
            )) {
                findings.push(relation_finding(
                    "relation.invalid",
                    &format!("edge '{}' target must be a string or table", raw.kind),
                    &raw.from,
                    raw.source.clone(),
                ));
            }
            None
        }
    }
}

/// Aggregate resolved edges into the kind-level architecture graph.
pub fn kind_graph(edges: &[ComponentEdge], flat: &[FlatInstance]) -> Vec<KindEdge> {
    let by_id: BTreeMap<&str, &FlatInstance> = flat.iter().map(|f| (f.id.as_str(), f)).collect();
    let mut counts: BTreeMap<(String, String, String, bool), usize> = BTreeMap::new();
    for edge in edges {
        let Some(from) = by_id.get(edge.from.as_str()) else {
            continue;
        };
        let (to, external) = match (&edge.to, &edge.symbol) {
            (Some(to_id), _) => match by_id.get(to_id.as_str()) {
                Some(target) => (target.component_id.clone(), false),
                None => continue,
            },
            (None, Some(symbol)) => (symbol.clone(), true),
            (None, None) => continue,
        };
        *counts
            .entry((from.component_id.clone(), edge.kind.clone(), to, external))
            .or_default() += 1;
    }
    counts
        .into_iter()
        .map(|((from, kind, to, external), count)| KindEdge {
            from,
            kind,
            to,
            count,
            external,
        })
        .collect()
}

fn relation_finding(
    code: &str,
    message: &str,
    instance_id: &str,
    locator: Option<serde_json::Value>,
) -> ComponentFinding {
    ComponentFinding {
        severity: "warning".to_string(),
        code: code.to_string(),
        message: message.to_string(),
        instance_id: Some(instance_id.to_string()),
        locator,
    }
}

fn relation_info_finding(
    code: &str,
    message: &str,
    instance_id: &str,
    locator: Option<serde_json::Value>,
) -> ComponentFinding {
    ComponentFinding {
        severity: "info".to_string(),
        ..relation_finding(code, message, instance_id, locator)
    }
}

