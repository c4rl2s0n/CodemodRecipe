//! Instance-graph Mermaid rendering.

//! Mermaid rendering of the relations graph: kind-level (architecture roll-up) and
//! instance-level (connected nodes + membership subgraphs). Pure functions over scan
//! data; output is deterministic given deterministic inputs.

use std::collections::{BTreeMap, BTreeSet};

use crate::component::membership::{validate_membership_forest, MembershipForest};
use crate::component::relations::{ComponentEdge, FlatInstance, KindEdge};

/// Mermaid-safe node id: every non-alphanumeric becomes `_`.
use super::style::{mermaid_id, mermaid_label};

pub fn render_instance_graph_mermaid(
    edges: &[ComponentEdge],
    flat: &[FlatInstance],
    member_kinds: &BTreeSet<String>,
) -> String {
    let (forest, _) = validate_membership_forest(edges, member_kinds);
    render_instance_graph_mermaid_with_forest(edges, flat, member_kinds, &forest)
}

fn render_instance_graph_mermaid_with_forest(
    edges: &[ComponentEdge],
    flat: &[FlatInstance],
    member_kinds: &BTreeSet<String>,
    forest: &MembershipForest,
) -> String {
    let arch_edges: Vec<&ComponentEdge> = edges
        .iter()
        .filter(|e| !member_kinds.contains(&e.kind))
        .collect();

    let mut incident: BTreeSet<&str> = BTreeSet::new();
    for edge in &arch_edges {
        incident.insert(edge.from.as_str());
        if let Some(to) = &edge.to {
            incident.insert(to.as_str());
        }
        if let Some(symbol) = &edge.symbol {
            incident.insert(symbol.as_str());
        }
    }
    for child in forest.parent.keys() {
        incident.insert(child.as_str());
    }

    let flat_ids: BTreeSet<&str> = flat.iter().map(|f| f.id.as_str()).collect();
    let visible_instances: Vec<&FlatInstance> = flat
        .iter()
        .filter(|f| incident.contains(f.id.as_str()))
        .collect();

    // Symbol leaves from architecture edges only.
    let mut symbols: BTreeMap<&str, ()> = BTreeMap::new();
    for edge in &arch_edges {
        if let Some(symbol) = &edge.symbol {
            if !flat_ids.contains(symbol.as_str()) {
                symbols.entry(symbol.as_str()).or_insert(());
            }
        }
    }

    let mut out = String::from("graph LR\n");

    // Nested subgraphs: keys that are themselves member children nest under their parent.
    let mut children_of: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
    let mut all_keys: BTreeSet<&str> = BTreeSet::new();
    for (child, parent) in &forest.parent {
        children_of.entry(parent.as_str()).or_default().push(child.as_str());
        all_keys.insert(parent.as_str());
    }
    for kids in children_of.values_mut() {
        kids.sort();
    }

    let mut root_keys: Vec<&str> = all_keys
        .iter()
        .copied()
        .filter(|k| !forest.parent.contains_key(*k))
        .collect();
    root_keys.sort();

    let mut emitted_nodes: BTreeSet<String> = BTreeSet::new();

    fn emit_subgraph(
        out: &mut String,
        key: &str,
        children_of: &BTreeMap<&str, Vec<&str>>,
        flat_by_id: &BTreeMap<&str, &FlatInstance>,
        emitted_nodes: &mut BTreeSet<String>,
        indent: usize,
    ) {
        let pad = "  ".repeat(indent);
        out.push_str(&format!(
            "{pad}subgraph {} [\"{}\"]\n",
            mermaid_id(key),
            mermaid_label(key)
        ));
        if let Some(members) = children_of.get(key) {
            for member in members {
                if children_of.contains_key(member) {
                    emit_subgraph(
                        out,
                        member,
                        children_of,
                        flat_by_id,
                        emitted_nodes,
                        indent + 1,
                    );
                }
                if let Some(inst) = flat_by_id.get(member) {
                    if emitted_nodes.insert(inst.id.clone()) {
                        let ipad = "  ".repeat(indent + 1);
                        out.push_str(&format!(
                            "{ipad}{}[\"{}\"]\n",
                            mermaid_id(&inst.id),
                            mermaid_label(&inst.id),
                        ));
                    }
                }
            }
        }
        out.push_str(&format!("{pad}end\n"));
    }

    let flat_by_id: BTreeMap<&str, &FlatInstance> =
        visible_instances.iter().map(|f| (f.id.as_str(), *f)).collect();

    for key in &root_keys {
        emit_subgraph(
            &mut out,
            key,
            &children_of,
            &flat_by_id,
            &mut emitted_nodes,
            1,
        );
    }

    // Remaining visible instances (no membership) at top level.
    for inst in &visible_instances {
        if emitted_nodes.insert(inst.id.clone()) {
            out.push_str(&format!(
                "  {}[\"{}\"]\n",
                mermaid_id(&inst.id),
                mermaid_label(&inst.id),
            ));
        }
    }

    for symbol in symbols.keys() {
        out.push_str(&format!(
            "  {}[\"{} (external)\"]\n",
            mermaid_id(symbol),
            mermaid_label(symbol),
        ));
    }

    for edge in arch_edges {
        let target = edge
            .to
            .as_deref()
            .or(edge.symbol.as_deref())
            .unwrap_or_default();
        if target.is_empty() {
            continue;
        }
        out.push_str(&format!(
            "  {} -- \"{}\" --> {}\n",
            mermaid_id(&edge.from),
            mermaid_label(&edge.kind),
            mermaid_id(target),
        ));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::relations::{flatten_instances, kind_graph, resolve_edges, RawEdge};
    use crate::component::types::ComponentInstance;

    fn instance(id: &str, component_id: &str, ast: serde_json::Value) -> ComponentInstance {
        ComponentInstance {
            id: id.to_string(),
            component_id: component_id.to_string(),
            locator: serde_json::json!({ "path": "lib/x.dart" }),
            ast,
            summary: None,
        }
    }

    fn sample_graph() -> (Vec<FlatInstance>, Vec<ComponentEdge>) {
        let instances = vec![
            instance(
                "app.model/recipe",
                "app.model",
                serde_json::json!({ "id": "recipe" }),
            ),
            instance(
                "app.store/recipe",
                "app.store",
                serde_json::json!({ "id": "recipe" }),
            ),
            instance(
                "app.store/user",
                "app.store",
                serde_json::json!({ "id": "user" }),
            ),
        ];
        let flat = flatten_instances(&instances);
        let raw = vec![
            RawEdge {
                from: "app.store/recipe".to_string(),
                kind: "stores".to_string(),
                to: Some(serde_json::json!("app.model/recipe")),
                symbol: None,
                source: None,
                extra: BTreeMap::new(),
            },
            RawEdge {
                from: "app.store/user".to_string(),
                kind: "stores".to_string(),
                to: Some(serde_json::json!("app.model/recipe")),
                symbol: None,
                source: None,
                extra: BTreeMap::new(),
            },
            RawEdge {
                from: "app.store/recipe".to_string(),
                kind: "extends".to_string(),
                to: None,
                symbol: Some("dart:core/Store".to_string()),
                source: None,
                extra: BTreeMap::new(),
            },
        ];
        let (edges, findings) = resolve_edges(raw, &flat, &Default::default());
        assert!(findings.is_empty());
        (flat, edges)
    }

    #[test]
    fn kind_graph_mermaid_rolls_up_with_counts() {
        let (flat, edges) = sample_graph();
        let kg = kind_graph(&edges, &flat);
        let mermaid = render_kind_graph_mermaid(&kg);
        assert!(
            mermaid
                .contains("app_store[\"app.store\"] -- \"stores ×2\" --> app_model[\"app.model\"]"),
            "kind edge with count: {mermaid}"
        );
        assert!(
            mermaid.contains(
                "app_store[\"app.store\"] -- \"extends\" --> dart_core_Store[\"dart:core/Store (external)\"]"
            ),
            "external symbol target: {mermaid}"
        );
        assert!(mermaid.starts_with("graph LR\n"));
    }

    #[test]
    fn instance_graph_omits_isolated_and_draws_edges() {
        let (flat, edges) = sample_graph();
        let mermaid = render_instance_graph_mermaid(&edges, &flat, &BTreeSet::new());
        assert!(
            mermaid.contains("app_model_recipe[\"app.model/recipe\"]"),
            "instance node: {mermaid}"
        );
        assert!(
            mermaid.contains("app_store_recipe -- \"stores\" --> app_model_recipe"),
            "instance edge: {mermaid}"
        );
        assert!(
            mermaid.contains("dart_core_Store[\"dart:core/Store (external)\"]"),
            "symbol leaf: {mermaid}"
        );
        // All sample nodes have edges — still present.
        assert!(mermaid.contains("app_store_user[\"app.store/user\"]"));
    }

    #[test]
    fn instance_graph_omits_truly_isolated() {
        let instances = vec![
            instance("a/1", "a", serde_json::json!({ "id": "1" })),
            instance("b/1", "b", serde_json::json!({ "id": "1" })),
            instance("lonely/1", "lonely", serde_json::json!({ "id": "1" })),
        ];
        let flat = flatten_instances(&instances);
        let raw = vec![RawEdge {
            from: "a/1".to_string(),
            kind: "uses".to_string(),
            to: Some(serde_json::json!("b/1")),
            symbol: None,
            source: None,
            extra: BTreeMap::new(),
        }];
        let (edges, _) = resolve_edges(raw, &flat, &Default::default());
        let mermaid = render_instance_graph_mermaid(&edges, &flat, &BTreeSet::new());
        assert!(!mermaid.contains("lonely"), "isolated omitted: {mermaid}");
        assert!(mermaid.contains("a_1"));
        assert!(mermaid.contains("b_1"));
    }

    #[test]
    fn membership_subgraph_and_no_member_arrow() {
        let instances = vec![
            instance("a/1", "a", serde_json::json!({ "id": "1" })),
            instance("b/1", "b", serde_json::json!({ "id": "1" })),
        ];
        let flat = flatten_instances(&instances);
        let member_kinds = BTreeSet::from(["lives_in".to_string()]);
        let raw = vec![
            RawEdge {
                from: "a/1".to_string(),
                kind: "lives_in".to_string(),
                to: Some(serde_json::json!("rust.runtime")),
                symbol: None,
                source: None,
                extra: BTreeMap::new(),
            },
            RawEdge {
                from: "a/1".to_string(),
                kind: "uses".to_string(),
                to: Some(serde_json::json!("b/1")),
                symbol: None,
                source: None,
                extra: BTreeMap::new(),
            },
            RawEdge {
                from: "b/1".to_string(),
                kind: "lives_in".to_string(),
                to: Some(serde_json::json!("rust.runtime")),
                symbol: None,
                source: None,
                extra: BTreeMap::new(),
            },
        ];
        let (edges, findings) = resolve_edges(raw, &flat, &member_kinds);
        assert!(findings.is_empty(), "{findings:?}");
        let mermaid = render_instance_graph_mermaid(&edges, &flat, &member_kinds);
        assert!(
            mermaid.contains("subgraph rust_runtime [\"rust.runtime\"]"),
            "{mermaid}"
        );
        assert!(!mermaid.contains("-- \"lives_in\" -->"), "{mermaid}");
        assert!(mermaid.contains("-- \"uses\" -->"), "{mermaid}");
    }
}
