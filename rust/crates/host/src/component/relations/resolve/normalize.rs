//! Normalize relate-hook return values into RawEdge lists.

use std::collections::BTreeMap;

use crate::component::types::ComponentFinding;

use super::findings::relation_finding;

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

