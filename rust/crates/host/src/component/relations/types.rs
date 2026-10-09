//! Flat instances and edge types for the relations graph.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::component::types::{context_path, ComponentInstance};

/// A resolved relation between two flat instances (or an external symbol).
/// Duplicates by `(from, kind, target)` merge into one canonical edge that always
/// carries `sources` (multiplicity is `sources.len()` — no stored count).
/// Extra fields from the component's edge table pass through onto the JSON.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentEdge {
    pub from: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<String>,
    pub sources: Vec<serde_json::Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
}

/// Kind-level aggregation entry: `(fromComponentId, kind, toComponentId)` + count.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KindEdge {
    pub from: String,
    pub kind: String,
    pub to: String,
    pub count: usize,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub external: bool,
}

/// A node in the instance forest with a `componentId`, flattened for graph phases.
#[derive(Debug, Clone)]
pub struct FlatInstance {
    pub id: String,
    pub component_id: String,
    /// Parse-hook payload (the AST projection); serialized as `ast`.
    pub ast: serde_json::Value,
    pub locator: serde_json::Value,
    pub parent_id: Option<String>,
    pub root_id: String,
}

impl FlatInstance {
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "componentId": self.component_id,
            "locator": self.locator,
            "ast": self.ast,
            "parentId": self.parent_id,
            "rootId": self.root_id,
        })
    }
}

/// Flat identity for child nodes: `id` → `name` → filename stem → `unknown`.
/// (Roots keep the original `instance_id` rule: `id` → stem → `unknown`.)
pub(super) fn child_instance_id(
    component_id: &str,
    node: &serde_json::Value,
    locator: &serde_json::Value,
) -> String {
    if let Some(id) = node.get("id").and_then(|v| v.as_str()) {
        return format!("{component_id}/{id}");
    }
    if let Some(name) = node.get("name").and_then(|v| v.as_str()) {
        return format!("{component_id}/{name}");
    }
    if let Some(path) = context_path(locator) {
        if let Some(stem) = crate::sandbox::path_template::stem_from_path(path) {
            return format!("{component_id}/{stem}");
        }
    }
    format!("{component_id}/unknown")
}

pub(super) fn child_locator(
    child: &serde_json::Value,
    parent_locator: &serde_json::Value,
) -> serde_json::Value {
    if let Some(source) = child.get("source") {
        return serde_json::json!({ "source": source });
    }
    parent_locator.clone()
}

/// Flatten the instance forest: every node with a `componentId` under `children`
/// (recursively) becomes a flat instance alongside the roots.
pub fn flatten_instances(instances: &[ComponentInstance]) -> Vec<FlatInstance> {
    let mut out = Vec::new();
    for instance in instances {
        flatten_node(
            &instance.ast,
            &instance.id,
            &instance.component_id,
            &instance.locator,
            None,
            &instance.id,
            &mut out,
        );
    }
    out
}

fn flatten_node(
    node: &serde_json::Value,
    id: &str,
    component_id: &str,
    locator: &serde_json::Value,
    parent_id: Option<String>,
    root_id: &str,
    out: &mut Vec<FlatInstance>,
) {
    out.push(FlatInstance {
        id: id.to_string(),
        component_id: component_id.to_string(),
        ast: node.clone(),
        locator: locator.clone(),
        parent_id: parent_id.clone(),
        root_id: root_id.to_string(),
    });
    if let Some(children) = node.get("children").and_then(|v| v.as_array()) {
        for child in children {
            let Some(child_component) = child.get("componentId").and_then(|v| v.as_str()) else {
                continue;
            };
            let locator = child_locator(child, locator);
            let child_id = child_instance_id(child_component, child, &locator);
            flatten_node(
                child,
                &child_id,
                child_component,
                &locator,
                Some(id.to_string()),
                root_id,
                out,
            );
        }
    }
}
