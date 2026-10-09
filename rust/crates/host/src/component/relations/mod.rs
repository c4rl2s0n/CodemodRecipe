//! Relations graph: flatten, resolve, attach, and Luau facades.

mod attach;
mod facade;
mod resolve;
mod types;

pub use attach::attach_relations;
pub use facade::{build_graph_facade, build_graph_facade_from_json, build_index_facade};
pub use resolve::{kind_graph, normalize_raw_edges, resolve_edges, RawEdge};
pub use types::{flatten_instances, ComponentEdge, FlatInstance, KindEdge};

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::*;
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

    #[test]
    fn flatten_includes_children_with_component_id() {
        let root = instance(
            "app.model/recipe",
            "app.model",
            serde_json::json!({
                "id": "recipe",
                "children": [
                    { "componentId": "app.property", "name": "title" },
                    { "componentId": "app.property", "name": "subtitle" },
                    { "name": "plain data node — skipped" }
                ]
            }),
        );
        let flat = flatten_instances(&[root]);
        let ids: Vec<&str> = flat.iter().map(|f| f.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "app.model/recipe",
                "app.property/title",
                "app.property/subtitle"
            ]
        );
        assert_eq!(flat[1].parent_id.as_deref(), Some("app.model/recipe"));
        assert_eq!(flat[1].root_id, "app.model/recipe");
    }

    #[test]
    fn resolve_edges_resolves_refs_and_merges_duplicates() {
        let flat = flatten_instances(&[
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
        ]);
        let raw = vec![
            RawEdge {
                from: "app.store/recipe".to_string(),
                kind: "stores".to_string(),
                to: Some(serde_json::json!({ "componentId": "app.model", "key": "recipe" })),
                symbol: None,
                source: None,
                extra: BTreeMap::new(),
            },
            RawEdge {
                from: "app.store/recipe".to_string(),
                kind: "stores".to_string(),
                to: Some(serde_json::json!("app.model/recipe")),
                symbol: None,
                source: None,
                extra: BTreeMap::new(),
            },
        ];
        let (edges, findings) = resolve_edges(raw, &flat, &Default::default());
        assert_eq!(edges.len(), 1, "duplicate edge must merge into one");
        assert_eq!(edges[0].to.as_deref(), Some("app.model/recipe"));
        assert!(edges[0].sources.is_empty(), "no provenance given");
        // Both duplicates lack provenance → identical → one info finding.
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "relation.duplicate");
        assert_eq!(findings[0].severity, "info");
    }

    #[test]
    fn resolve_edges_merges_distinct_provenance_silently() {
        let flat = flatten_instances(&[
            instance("app.model/a", "app.model", serde_json::json!({ "id": "a" })),
            instance("app.store/a", "app.store", serde_json::json!({ "id": "a" })),
        ]);
        let mk = |kind: &str, path: &str| RawEdge {
            from: "app.store/a".to_string(),
            kind: kind.to_string(),
            to: Some(serde_json::json!("app.model/a")),
            symbol: None,
            source: Some(serde_json::json!({ "path": path })),
            extra: BTreeMap::new(),
        };
        let (edges, findings) = resolve_edges(
            vec![
                mk("stores", "lib/one.dart"),
                mk("stores", "lib/two.dart"),
                // Different kind: never merges, even with identical provenance.
                mk("watches", "lib/one.dart"),
            ],
            &flat,
            &Default::default(),
        );
        assert_eq!(edges.len(), 2, "kind is part of the merge key");
        let stores = edges.iter().find(|e| e.kind == "stores").expect("stores");
        assert_eq!(stores.sources.len(), 2, "multiplicity is #sources");
        assert!(
            findings.is_empty(),
            "distinct provenance merges without findings"
        );
    }

    #[test]
    fn resolve_edges_flags_unresolved_refs() {
        let flat = flatten_instances([].as_slice());
        let raw = vec![RawEdge {
            from: "app.store/recipe".to_string(),
            kind: "stores".to_string(),
            to: Some(serde_json::json!({ "componentId": "app.model", "key": "ghost" })),
            symbol: None,
            source: None,
            extra: BTreeMap::new(),
        }];
        // from is unknown too, so we get the from-finding only
        let (edges, findings) = resolve_edges(raw, &flat, &Default::default());
        assert!(edges.is_empty());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "relation.unresolved");
    }

    #[test]
    fn kind_graph_aggregates_with_counts() {
        let flat = flatten_instances(&[
            instance("app.model/a", "app.model", serde_json::json!({ "id": "a" })),
            instance("app.model/b", "app.model", serde_json::json!({ "id": "b" })),
            instance("app.store/a", "app.store", serde_json::json!({ "id": "a" })),
            instance("app.store/b", "app.store", serde_json::json!({ "id": "b" })),
        ]);
        let mk = |from: &str, to: &str| RawEdge {
            from: from.to_string(),
            kind: "stores".to_string(),
            to: Some(serde_json::json!(to)),
            symbol: None,
            source: None,
            extra: BTreeMap::new(),
        };
        let (edges, _) = resolve_edges(
            vec![
                mk("app.store/a", "app.model/a"),
                mk("app.store/b", "app.model/b"),
            ],
            &flat,
            &Default::default(),
        );
        let graph = kind_graph(&edges, &flat);
        assert_eq!(graph.len(), 1);
        assert_eq!(graph[0].from, "app.store");
        assert_eq!(graph[0].to, "app.model");
        assert_eq!(graph[0].count, 2);
        assert!(!graph[0].external);
    }

    #[test]
    fn attach_relations_groups_by_kind_on_nodes() {
        let mut root = instance(
            "app.model/recipe",
            "app.model",
            serde_json::json!({
                "id": "recipe",
                "children": [ { "componentId": "app.property", "name": "title" } ]
            }),
        );
        let flat = flatten_instances(&[root.clone()]);
        let raw = vec![
            RawEdge {
                from: "app.model/recipe".to_string(),
                kind: "has".to_string(),
                to: Some(serde_json::json!("app.property/title")),
                symbol: None,
                source: Some(serde_json::json!({ "path": "lib/x.dart" })),
                extra: BTreeMap::from([("cardinality".to_string(), serde_json::json!("one"))]),
            },
            RawEdge {
                from: "app.model/recipe".to_string(),
                kind: "extends".to_string(),
                to: None,
                symbol: Some("dart:core/Object".to_string()),
                source: None,
                extra: BTreeMap::new(),
            },
        ];
        let (edges, findings) = resolve_edges(raw, &flat, &Default::default());
        assert!(findings.is_empty());
        attach_relations(std::slice::from_mut(&mut root), &edges);
        let relations = root.ast["relations"].as_object().expect("relations");
        assert_eq!(relations["has"][0]["to"], "app.property/title");
        assert_eq!(relations["has"][0]["cardinality"], "one");
        assert_eq!(relations["has"][0]["sources"][0]["path"], "lib/x.dart");
        assert_eq!(relations["extends"][0]["symbol"], "dart:core/Object");
        assert_eq!(
            relations["extends"][0]["sources"],
            serde_json::json!([]),
            "sources is always a list"
        );
    }
}
