//! Mermaid rendering of the relations graph: kind-level (architecture roll-up) and
//! instance-level (every flat node). Pure functions over scan data; output is
//! deterministic given deterministic inputs (kind graph is BTreeMap-ordered).

use std::collections::BTreeMap;

use crate::component::relations::{ComponentEdge, FlatInstance, KindEdge};

/// Mermaid-safe node id: every non-alphanumeric becomes `_`.
fn mermaid_id(raw: &str) -> String {
    raw.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// Escape a label for a quoted mermaid string.
fn mermaid_label(raw: &str) -> String {
    raw.replace('"', "#quot;")
}

/// Kind-level architecture graph: one line per `(fromComponent, kind, toComponent)`
/// roll-up, edge label `kind ×count` (the `×1` is omitted). External symbol targets
/// get `(external)` in their label.
pub fn render_kind_graph_mermaid(kind_graph: &[KindEdge]) -> String {
    let mut out = String::from("graph LR\n");
    for edge in kind_graph {
        let label = if edge.count > 1 {
            format!("{} ×{}", edge.kind, edge.count)
        } else {
            edge.kind.clone()
        };
        let to_label = if edge.external {
            format!("{} (external)", edge.to)
        } else {
            edge.to.clone()
        };
        out.push_str(&format!(
            "  {}[\"{}\"] -- \"{}\" --> {}[\"{}\"]\n",
            mermaid_id(&edge.from),
            mermaid_label(&edge.from),
            mermaid_label(&label),
            mermaid_id(&edge.to),
            mermaid_label(&to_label),
        ));
    }
    out
}

/// Instance-level graph: one node per flat instance (including isolated nodes), one
/// line per resolved edge; external symbol targets become leaf nodes.
pub fn render_instance_graph_mermaid(edges: &[ComponentEdge], flat: &[FlatInstance]) -> String {
    let mut out = String::from("graph LR\n");
    for inst in flat {
        out.push_str(&format!(
            "  {}[\"{}\"]\n",
            mermaid_id(&inst.id),
            mermaid_label(&inst.id),
        ));
    }
    // Symbol leaves are declared once, after the instances.
    let mut symbols: BTreeMap<&str, ()> = BTreeMap::new();
    for edge in edges {
        if let Some(symbol) = &edge.symbol {
            symbols.entry(symbol.as_str()).or_insert(());
        }
    }
    for symbol in symbols.keys() {
        out.push_str(&format!(
            "  {}[\"{} (external)\"]\n",
            mermaid_id(symbol),
            mermaid_label(symbol),
        ));
    }
    for edge in edges {
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
        let (edges, findings) = resolve_edges(raw, &flat);
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
    fn instance_graph_mermaid_lists_nodes_edges_and_symbol_leaves() {
        let (flat, edges) = sample_graph();
        let mermaid = render_instance_graph_mermaid(&edges, &flat);
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
        assert!(
            mermaid.contains("app_store_recipe -- \"extends\" --> dart_core_Store"),
            "symbol edge: {mermaid}"
        );
        // Isolated node (app.store/user has an outgoing edge but app.model/recipe
        // has none) — all instances appear regardless of connectivity.
        assert!(mermaid.contains("app_store_user[\"app.store/user\"]"));
    }
}
