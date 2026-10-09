//! Membership forest from relation kinds with `layout = "member"`.
//!
//! Member targets are opaque string subgraph keys (free labels or instance ids).
//! Multi-parent and cycles yield concrete `relation.*` findings; the forest is
//! still repaired deterministically so mermaid can render.

use std::collections::{BTreeMap, BTreeSet};

use codemod_recipe_lua::RelationKindDef;

use crate::component::relations::ComponentEdge;
use crate::component::types::ComponentFinding;

/// Child instance id → chosen parent subgraph key.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MembershipForest {
    pub parent: BTreeMap<String, String>,
}

/// Kinds whose stack registry layout is `member`.
pub fn member_kinds(relations: &BTreeMap<String, RelationKindDef>) -> BTreeSet<String> {
    relations
        .iter()
        .filter(|(_, def)| def.layout == "member")
        .map(|(kind, _)| kind.clone())
        .collect()
}

/// Parent key for a membership edge: `to` string, else `symbol`.
pub fn membership_parent_key(edge: &ComponentEdge) -> Option<String> {
    edge.to
        .clone()
        .or_else(|| edge.symbol.clone())
        .filter(|s| !s.is_empty())
}

/// Build a membership forest from resolved edges of member kinds.
///
/// On multi-membership, emits `relation.multi_membership` and keeps the
/// lexicographically first parent. On cycles, emits `relation.membership_cycle`
/// and breaks at the lexicographically smallest id on the cycle.
pub fn validate_membership_forest(
    edges: &[ComponentEdge],
    member_kinds: &BTreeSet<String>,
) -> (MembershipForest, Vec<ComponentFinding>) {
    let mut findings = Vec::new();
    let mut parents_by_child: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();

    for edge in edges {
        if !member_kinds.contains(&edge.kind) {
            continue;
        }
        let Some(parent) = membership_parent_key(edge) else {
            continue;
        };
        parents_by_child
            .entry(edge.from.clone())
            .or_default()
            .insert(parent);
    }

    let mut parent = BTreeMap::new();
    for (child, parents) in &parents_by_child {
        let mut ordered: Vec<&String> = parents.iter().collect();
        ordered.sort();
        if ordered.len() > 1 {
            findings.push(ComponentFinding {
                severity: "warning".to_string(),
                code: "relation.multi_membership".to_string(),
                message: format!(
                    "`{child}` belongs to both `{}` and `{}` via member kinds (exclusive containment)",
                    ordered[0], ordered[1]
                ),
                instance_id: Some(child.clone()),
                locator: None,
            });
        }
        if let Some(first) = ordered.first() {
            parent.insert(child.clone(), (*first).clone());
        }
    }

    // Break cycles: walk each child; if a cycle is found, drop the edge from the
    // lexicographically smallest id on the cycle.
    let children: Vec<String> = parent.keys().cloned().collect();
    for start in children {
        let mut path: Vec<String> = Vec::new();
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        let mut cur = start.clone();
        loop {
            if let Some(&idx) = seen.get(&cur) {
                let cycle = &path[idx..];
                let break_at = cycle.iter().min().cloned().unwrap_or_else(|| cur.clone());
                findings.push(ComponentFinding {
                    severity: "warning".to_string(),
                    code: "relation.membership_cycle".to_string(),
                    message: format!(
                        "membership cycle: {} → {}",
                        cycle.join(" → "),
                        cycle.first().map(String::as_str).unwrap_or("")
                    ),
                    instance_id: Some(break_at.clone()),
                    locator: None,
                });
                parent.remove(&break_at);
                break;
            }
            seen.insert(cur.clone(), path.len());
            path.push(cur.clone());
            let Some(next) = parent.get(&cur).cloned() else {
                break;
            };
            // Only continue the walk when the parent key is itself a member child
            // (instance used as nested subgraph key).
            if !parent.contains_key(&next) {
                break;
            }
            cur = next;
        }
    }

    (MembershipForest { parent }, findings)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn edge(from: &str, kind: &str, to: &str) -> ComponentEdge {
        ComponentEdge {
            from: from.to_string(),
            kind: kind.to_string(),
            to: Some(to.to_string()),
            symbol: None,
            sources: vec![],
            extra: BTreeMap::new(),
        }
    }

    #[test]
    fn multi_membership_warns_and_picks_first() {
        let kinds = BTreeSet::from(["lives_in".to_string()]);
        let edges = vec![
            edge("a", "lives_in", "g2"),
            edge("a", "lives_in", "g1"),
        ];
        let (forest, findings) = validate_membership_forest(&edges, &kinds);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].code, "relation.multi_membership");
        assert!(findings[0].message.contains("`g1`"));
        assert!(findings[0].message.contains("`g2`"));
        assert_eq!(forest.parent.get("a").map(String::as_str), Some("g1"));
    }

    #[test]
    fn cycle_warns_and_breaks() {
        let kinds = BTreeSet::from(["lives_in".to_string()]);
        let edges = vec![
            edge("a", "lives_in", "b"),
            edge("b", "lives_in", "a"),
        ];
        let (forest, findings) = validate_membership_forest(&edges, &kinds);
        assert!(
            findings.iter().any(|f| f.code == "relation.membership_cycle"),
            "{findings:?}"
        );
        // Exactly one parent edge remains after break.
        assert_eq!(forest.parent.len(), 1, "{:?}", forest.parent);
    }

    #[test]
    fn longer_cycle() {
        let kinds = BTreeSet::from(["lives_in".to_string()]);
        let edges = vec![
            edge("a", "lives_in", "b"),
            edge("b", "lives_in", "c"),
            edge("c", "lives_in", "a"),
        ];
        let (_forest, findings) = validate_membership_forest(&edges, &kinds);
        assert!(findings.iter().any(|f| f.code == "relation.membership_cycle"));
    }

    #[test]
    fn valid_nested_tree_no_findings() {
        let kinds = BTreeSet::from(["lives_in".to_string()]);
        let edges = vec![
            edge("child", "lives_in", "parent_inst"),
            edge("parent_inst", "lives_in", "rust.runtime"),
        ];
        let (forest, findings) = validate_membership_forest(&edges, &kinds);
        assert!(findings.is_empty(), "{findings:?}");
        assert_eq!(
            forest.parent.get("child").map(String::as_str),
            Some("parent_inst")
        );
        assert_eq!(
            forest.parent.get("parent_inst").map(String::as_str),
            Some("rust.runtime")
        );
    }

    #[test]
    fn undeclared_kind_ignored() {
        let kinds = BTreeSet::from(["lives_in".to_string()]);
        let edges = vec![edge("a", "invokes", "b")];
        let (forest, findings) = validate_membership_forest(&edges, &kinds);
        assert!(findings.is_empty());
        assert!(forest.parent.is_empty());
    }
}
