//! Resolve edge targets (member keys + architecture refs).

use std::collections::{BTreeMap, BTreeSet};

use crate::relations::types::FlatInstance;
use crate::types::ComponentFinding;

use super::findings::relation_finding;
use super::RawEdge;

/// Coerce a member-kind target to an opaque subgraph key string.
pub(super) fn member_parent_key(
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

pub(super) fn resolve_architecture_target(
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

