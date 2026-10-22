//! Attach resolved edges onto instance AST nodes.

use std::collections::BTreeMap;

use crate::relations::types::{child_instance_id, child_locator, ComponentEdge};
use crate::types::ComponentInstance;

/// Attach outgoing edges onto instance nodes as `node.relations` (grouped by kind),
/// replacing any previously attached relations.
pub fn attach_relations(instances: &mut [ComponentInstance], edges: &[ComponentEdge]) {
    let mut by_from: BTreeMap<&str, BTreeMap<String, Vec<serde_json::Value>>> = BTreeMap::new();
    for edge in edges {
        let mut entry = serde_json::Map::new();
        if let Some(to) = &edge.to {
            entry.insert("to".to_string(), serde_json::Value::String(to.clone()));
        }
        if let Some(symbol) = &edge.symbol {
            entry.insert(
                "symbol".to_string(),
                serde_json::Value::String(symbol.clone()),
            );
        }
        entry.insert(
            "sources".to_string(),
            serde_json::Value::Array(edge.sources.clone()),
        );
        for (key, value) in &edge.extra {
            entry.insert(key.clone(), value.clone());
        }
        by_from
            .entry(edge.from.as_str())
            .or_default()
            .entry(edge.kind.clone())
            .or_default()
            .push(serde_json::Value::Object(entry));
    }

    for instance in instances {
        let root_id = instance.id.clone();
        let locator = instance.locator.clone();
        attach_node_relations(&mut instance.ast, &root_id, &locator, &by_from);
    }
}

fn attach_node_relations(
    node: &mut serde_json::Value,
    id: &str,
    locator: &serde_json::Value,
    by_from: &BTreeMap<&str, BTreeMap<String, Vec<serde_json::Value>>>,
) {
    if let Some(obj) = node.as_object_mut() {
        match by_from.get(id) {
            Some(kinds) => {
                let relations: serde_json::Map<String, serde_json::Value> = kinds
                    .iter()
                    .map(|(kind, entries)| {
                        (kind.clone(), serde_json::Value::Array(entries.clone()))
                    })
                    .collect();
                obj.insert(
                    "relations".to_string(),
                    serde_json::Value::Object(relations),
                );
            }
            None => {
                obj.remove("relations");
            }
        }
        if let Some(children) = obj.get_mut("children").and_then(|v| v.as_array_mut()) {
            for child in children {
                let Some(child_component) = child.get("componentId").and_then(|v| v.as_str())
                else {
                    continue;
                };
                let child_locator = child_locator(child, locator);
                let child_id = child_instance_id(child_component, child, &child_locator);
                attach_node_relations(child, &child_id, &child_locator, by_from);
            }
        }
    }
}

