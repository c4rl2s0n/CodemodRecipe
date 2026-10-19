//! Relation coverage metrics for scan snapshots.
//!
//! Counts membership vs architecture connectivity over the flattened instance
//! forest, applying `stack.zoneRoles` flags (`ignoreRelations`).
//!
//! Architecture ratio eligibility (inferred, no component role field):
//! - Zone-rooted / free-root instances are eligible.
//! - Nested under a non-zone parent are eligible only if they participate in a
//!   non-membership edge (`uses` / `feeds` / …). Membership-only children are
//!   structural/span members — not architecture-isolated units.

use std::collections::{BTreeMap, BTreeSet};

use codemod_recipe_lua::{RelationKindDef, ZoneDef, ZoneRole};
use serde::Serialize;

use crate::component::membership::{
    member_kinds, validate_membership_forest, MembershipForest,
};
use crate::component::relations::{flatten_instances, ComponentEdge, FlatInstance};
use crate::component::types::ComponentInstance;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationRatios {
    pub connected: f64,
    /// Connected among [`RelationAggregate::architecture_eligible`] only.
    pub architecture_connected: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationAggregate {
    pub instance_count: usize,
    pub edge_count: usize,
    pub isolated: usize,
    pub membership_connected: usize,
    pub architecture_connected: usize,
    pub architecture_isolated: usize,
    /// Instances in the architecture ratio denominator (see module docs).
    pub architecture_eligible: usize,
    pub relation_ignored: usize,
    pub ratios: RelationRatios,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationByComponent {
    pub component_id: String,
    pub instance_count: usize,
    pub architecture_connected: usize,
    pub architecture_isolated: usize,
    pub architecture_eligible: usize,
    pub membership_only: usize,
    pub relation_ignored: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationMetrics {
    pub aggregate: RelationAggregate,
    pub by_component: Vec<RelationByComponent>,
    pub by_kind: BTreeMap<String, usize>,
}

/// Instance ids under a zone whose role has `ignoreCoverage` (for compact snapshot).
pub fn ignore_coverage_instance_ids(
    instances: &[ComponentInstance],
    edges: &[ComponentEdge],
    relations: &BTreeMap<String, RelationKindDef>,
    zones: &[ZoneDef],
    zone_roles: &BTreeMap<String, ZoneRole>,
) -> BTreeSet<String> {
    let flat = flatten_instances(instances);
    let member = member_kinds(relations);
    let (forest, _) = validate_membership_forest(edges, &member);
    let zone_inst = zone_instance_roles(&flat, zones);
    let mut out = BTreeSet::new();
    for inst in &flat {
        let flags = flags_for_member(&inst.id, &forest, &zone_inst, zone_roles);
        if flags.ignore_coverage {
            out.insert(inst.id.clone());
        }
    }
    out
}

/// Compute relation metrics for a scan forest.
pub fn relation_metrics(
    instances: &[ComponentInstance],
    edges: &[ComponentEdge],
    relations: &BTreeMap<String, RelationKindDef>,
    zones: &[ZoneDef],
    zone_roles: &BTreeMap<String, ZoneRole>,
) -> RelationMetrics {
    let flat = flatten_instances(instances);
    let member = member_kinds(relations);
    let (forest, _) = validate_membership_forest(edges, &member);
    let zone_inst = zone_instance_roles(&flat, zones);

    let mut by_kind: BTreeMap<String, usize> = BTreeMap::new();
    for edge in edges {
        *by_kind.entry(edge.kind.clone()).or_default() += 1;
    }

    // Per-instance connectivity
    let mut any_deg: BTreeSet<String> = BTreeSet::new();
    let mut member_deg: BTreeSet<String> = BTreeSet::new();
    let mut arch_deg: BTreeSet<String> = BTreeSet::new();
    for edge in edges {
        any_deg.insert(edge.from.clone());
        let is_member = member.contains(&edge.kind);
        if is_member {
            member_deg.insert(edge.from.clone());
        } else {
            arch_deg.insert(edge.from.clone());
            // External symbol targets count for from only; resolved `to` counts both.
            if let Some(to) = edge.to.as_ref() {
                any_deg.insert(to.clone());
                arch_deg.insert(to.clone());
            }
        }
    }

    let mut agg_isolated = 0usize;
    let mut agg_membership = 0usize;
    let mut agg_arch = 0usize;
    let mut agg_arch_isolated = 0usize;
    let mut agg_eligible = 0usize;
    let mut agg_ignored = 0usize;

    let mut by_comp: BTreeMap<String, RelationByComponent> = BTreeMap::new();

    for inst in &flat {
        let flags = flags_for_member(&inst.id, &forest, &zone_inst, zone_roles);
        let ignored = flags.ignore_relations;
        let connected = any_deg.contains(&inst.id);
        let membership = member_deg.contains(&inst.id);
        let architecture = arch_deg.contains(&inst.id);
        let membership_only = connected && membership && !architecture;
        let nested = nested_under_unit(&inst.id, &forest, &zone_inst);
        let eligible = architecture_eligible(ignored, nested, architecture);
        // Isolated only among architecture-eligible instances.
        let architecture_isolated = eligible && !architecture;

        if !connected {
            agg_isolated += 1;
        }
        if membership {
            agg_membership += 1;
        }
        if eligible {
            agg_eligible += 1;
            if architecture {
                agg_arch += 1;
            }
            if architecture_isolated {
                agg_arch_isolated += 1;
            }
        }
        if ignored {
            agg_ignored += 1;
        }

        let entry = by_comp
            .entry(inst.component_id.clone())
            .or_insert_with(|| RelationByComponent {
                component_id: inst.component_id.clone(),
                instance_count: 0,
                architecture_connected: 0,
                architecture_isolated: 0,
                architecture_eligible: 0,
                membership_only: 0,
                relation_ignored: 0,
            });
        entry.instance_count += 1;
        if eligible {
            entry.architecture_eligible += 1;
            if architecture {
                entry.architecture_connected += 1;
            }
            if architecture_isolated {
                entry.architecture_isolated += 1;
            }
        }
        if membership_only {
            entry.membership_only += 1;
        }
        if ignored {
            entry.relation_ignored += 1;
        }
    }

    let n = flat.len();
    let connected_count = n.saturating_sub(agg_isolated);
    let ratios = RelationRatios {
        connected: if n == 0 {
            1.0
        } else {
            connected_count as f64 / n as f64
        },
        architecture_connected: if agg_eligible == 0 {
            1.0
        } else {
            agg_arch as f64 / agg_eligible as f64
        },
    };

    let mut by_component: Vec<_> = by_comp.into_values().collect();
    by_component.sort_by(|a, b| a.component_id.cmp(&b.component_id));

    RelationMetrics {
        aggregate: RelationAggregate {
            instance_count: n,
            edge_count: edges.len(),
            isolated: agg_isolated,
            membership_connected: agg_membership,
            architecture_connected: agg_arch,
            architecture_isolated: agg_arch_isolated,
            architecture_eligible: agg_eligible,
            relation_ignored: agg_ignored,
            ratios,
        },
        by_component,
        by_kind,
    }
}

/// Membership parent exists and is not a zone instance → nested under a model unit.
fn nested_under_unit(
    instance_id: &str,
    forest: &MembershipForest,
    zone_inst: &BTreeMap<String, String>,
) -> bool {
    match forest.parent.get(instance_id) {
        Some(parent) => !zone_inst.contains_key(parent),
        None => false,
    }
}

fn architecture_eligible(ignored: bool, nested_under_unit: bool, has_arch_edge: bool) -> bool {
    !ignored && (!nested_under_unit || has_arch_edge)
}

/// Map zone catalog id → flat instance id, and instance id → role string.
fn zone_instance_roles(
    flat: &[FlatInstance],
    zones: &[ZoneDef],
) -> BTreeMap<String, String> {
    let zone_ids: BTreeSet<&str> = zones.iter().map(|z| z.id.as_str()).collect();
    let mut role_by_zone_id: BTreeMap<&str, &str> = BTreeMap::new();
    for z in zones {
        if let Some(role) = z.role.as_deref() {
            role_by_zone_id.insert(z.id.as_str(), role);
        }
    }

    let mut out = BTreeMap::new();
    for inst in flat {
        let Some(ast_id) = inst.ast.get("id").and_then(|v| v.as_str()) else {
            continue;
        };
        if !zone_ids.contains(ast_id) {
            continue;
        }
        // Prefer catalog role; fall back to ast.role.
        let role = role_by_zone_id
            .get(ast_id)
            .copied()
            .or_else(|| inst.ast.get("role").and_then(|v| v.as_str()));
        if let Some(role) = role {
            out.insert(inst.id.clone(), role.to_string());
        }
    }
    out
}

/// Walk membership parents; use the outermost zone ancestor's role flags.
fn flags_for_member(
    instance_id: &str,
    forest: &MembershipForest,
    zone_inst: &BTreeMap<String, String>,
    zone_roles: &BTreeMap<String, ZoneRole>,
) -> ZoneRole {
    let mut cur = instance_id;
    let mut found_role: Option<&str> = None;
    // Walk toward root; keep updating when we see a zone so outermost wins.
    loop {
        let Some(parent) = forest.parent.get(cur) else {
            break;
        };
        if let Some(role) = zone_inst.get(parent) {
            found_role = Some(role.as_str());
        }
        if !forest.parent.contains_key(parent) {
            break;
        }
        cur = parent.as_str();
    }
    match found_role {
        Some(role) => zone_roles.get(role).cloned().unwrap_or_default(),
        None => ZoneRole::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::types::ComponentInstance;

    fn inst(id: &str, component_id: &str) -> ComponentInstance {
        ComponentInstance {
            id: id.to_string(),
            component_id: component_id.to_string(),
            locator: serde_json::json!({}),
            ast: serde_json::json!({ "id": id.rsplit('/').next().unwrap_or(id) }),
            summary: None,
        }
    }

    fn zone_inst(id: &str, role: &str) -> ComponentInstance {
        ComponentInstance {
            id: format!("repo.zone/{id}"),
            component_id: "repo.zone".to_string(),
            locator: serde_json::json!({ "id": id }),
            ast: serde_json::json!({ "id": id, "role": role, "title": id }),
            summary: None,
        }
    }

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

    fn relations() -> BTreeMap<String, RelationKindDef> {
        BTreeMap::from([(
            "lives_in".to_string(),
            RelationKindDef {
                layout: "member".to_string(),
            },
        )])
    }

    #[test]
    fn ignore_relations_exempts_structural_members() {
        let zones = vec![
            ZoneDef {
                id: "structural.modules".to_string(),
                role: Some("structural".to_string()),
                title: Some("Modules".to_string()),
                parent_id: None,
            },
            ZoneDef {
                id: "command.adapters".to_string(),
                role: Some("architecture".to_string()),
                title: Some("Adapters".to_string()),
                parent_id: None,
            },
        ];
        let zone_roles = BTreeMap::from([(
            "structural".to_string(),
            ZoneRole {
                ignore_relations: true,
                ignore_coverage: false,
            },
        )]);

        let instances = vec![
            zone_inst("structural.modules", "structural"),
            zone_inst("command.adapters", "architecture"),
            inst("repo.barrel/a", "repo.barrel"),
            inst("repo.cmd/x", "repo.cmd"),
            inst("repo.engine/e", "repo.engine"),
        ];
        let edges = vec![
            edge(
                "repo.barrel/a",
                "lives_in",
                "repo.zone/structural.modules",
            ),
            edge("repo.cmd/x", "lives_in", "repo.zone/command.adapters"),
            edge("repo.cmd/x", "uses", "repo.engine/e"),
        ];

        let m = relation_metrics(&instances, &edges, &relations(), &zones, &zone_roles);
        assert_eq!(m.aggregate.instance_count, 5);
        assert_eq!(m.aggregate.relation_ignored, 1, "{:?}", m.aggregate);
        assert_eq!(m.aggregate.architecture_connected, 2); // cmd + engine
        // barrel is membership-only + ignored → not architectureIsolated
        let barrel = m
            .by_component
            .iter()
            .find(|c| c.component_id == "repo.barrel")
            .unwrap();
        assert_eq!(barrel.relation_ignored, 1);
        assert_eq!(barrel.architecture_isolated, 0);
        assert_eq!(barrel.membership_only, 1);
        // architecture ratio denom = eligible (zones + cmd + engine; barrel ignored)
        assert_eq!(m.aggregate.architecture_eligible, 4, "{:?}", m.aggregate);
        let expected = m.aggregate.architecture_connected as f64
            / m.aggregate.architecture_eligible as f64;
        assert!((m.aggregate.ratios.architecture_connected - expected).abs() < 1e-9);
    }

    #[test]
    fn title_only_role_does_not_ignore() {
        let zones = vec![ZoneDef {
            id: "docs.surface".to_string(),
            role: Some("documentation".to_string()),
            title: Some("Docs".to_string()),
            parent_id: None,
        }];
        let zone_roles = BTreeMap::new(); // no entry for documentation

        let instances = vec![
            zone_inst("docs.surface", "documentation"),
            inst("repo.doc/a", "repo.doc"),
        ];
        let edges = vec![edge(
            "repo.doc/a",
            "lives_in",
            "repo.zone/docs.surface",
        )];
        let m = relation_metrics(&instances, &edges, &relations(), &zones, &zone_roles);
        assert_eq!(m.aggregate.relation_ignored, 0);
        // zone-rooted member is eligible; membership-only → architectureIsolated
        assert_eq!(m.aggregate.architecture_eligible, 2);
        assert_eq!(m.aggregate.architecture_isolated, 2); // zone + member
    }

    #[test]
    fn nested_membership_only_child_not_architecture_eligible() {
        let zones = vec![ZoneDef {
            id: "domain.recipe".to_string(),
            role: Some("architecture".to_string()),
            title: Some("Recipe".to_string()),
            parent_id: None,
        }];
        let zone_roles = BTreeMap::new();
        let instances = vec![
            zone_inst("domain.recipe", "architecture"),
            inst("repo.host_module/recipe", "repo.host_module"),
            inst("repo.host_module_item/fn@1", "repo.host_module_item"),
        ];
        let edges = vec![
            edge(
                "repo.host_module/recipe",
                "lives_in",
                "repo.zone/domain.recipe",
            ),
            edge(
                "repo.host_module_item/fn@1",
                "lives_in",
                "repo.host_module/recipe",
            ),
        ];
        let m = relation_metrics(&instances, &edges, &relations(), &zones, &zone_roles);
        let item = m
            .by_component
            .iter()
            .find(|c| c.component_id == "repo.host_module_item")
            .unwrap();
        assert_eq!(item.membership_only, 1);
        assert_eq!(item.architecture_eligible, 0);
        assert_eq!(item.architecture_isolated, 0);
        // zone + host_module eligible; item excluded
        assert_eq!(m.aggregate.architecture_eligible, 2, "{:?}", m.aggregate);
        assert_eq!(m.aggregate.architecture_isolated, 2);
        assert_eq!(m.aggregate.architecture_connected, 0);
        assert!((m.aggregate.ratios.architecture_connected - 0.0).abs() < 1e-9);
    }

    #[test]
    fn nested_child_with_architecture_edge_is_eligible_and_connected() {
        let zones = vec![ZoneDef {
            id: "domain.host_api".to_string(),
            role: Some("architecture".to_string()),
            title: Some("Host API".to_string()),
            parent_id: None,
        }];
        let zone_roles = BTreeMap::new();
        let instances = vec![
            zone_inst("domain.host_api", "architecture"),
            inst("repo.host_module/host_api", "repo.host_module"),
            inst("repo.host_module_item/do_query@1", "repo.host_module_item"),
            inst("repo.engine_edit/engine-edit", "repo.engine_edit"),
        ];
        let edges = vec![
            edge(
                "repo.host_module/host_api",
                "lives_in",
                "repo.zone/domain.host_api",
            ),
            edge(
                "repo.engine_edit/engine-edit",
                "lives_in",
                "repo.zone/domain.host_api",
            ),
            edge(
                "repo.host_module_item/do_query@1",
                "lives_in",
                "repo.host_module/host_api",
            ),
            edge(
                "repo.host_module_item/do_query@1",
                "drives",
                "repo.engine_edit/engine-edit",
            ),
        ];
        let mut rel = relations();
        rel.insert(
            "drives".to_string(),
            RelationKindDef {
                layout: "edge".to_string(),
            },
        );
        let m = relation_metrics(&instances, &edges, &rel, &zones, &zone_roles);
        let item = m
            .by_component
            .iter()
            .find(|c| c.component_id == "repo.host_module_item")
            .unwrap();
        assert_eq!(item.architecture_eligible, 1);
        assert_eq!(item.architecture_connected, 1);
        assert_eq!(item.architecture_isolated, 0);
        // item + engine_edit connected; zone + host_module eligible isolated
        assert_eq!(m.aggregate.architecture_connected, 2, "{:?}", m.aggregate);
        assert_eq!(m.aggregate.architecture_eligible, 4);
    }

    #[test]
    fn ignore_coverage_ids_include_zone_members() {
        let zones = vec![ZoneDef {
            id: "noise".to_string(),
            role: Some("generated".to_string()),
            title: None,
            parent_id: None,
        }];
        let zone_roles = BTreeMap::from([(
            "generated".to_string(),
            ZoneRole {
                ignore_relations: false,
                ignore_coverage: true,
            },
        )]);
        let instances = vec![
            zone_inst("noise", "generated"),
            inst("repo.gen/a", "repo.gen"),
            inst("repo.keep/b", "repo.keep"),
        ];
        let edges = vec![edge("repo.gen/a", "lives_in", "repo.zone/noise")];
        let ids = ignore_coverage_instance_ids(
            &instances,
            &edges,
            &relations(),
            &zones,
            &zone_roles,
        );
        assert!(ids.contains("repo.gen/a"), "{ids:?}");
        assert!(!ids.contains("repo.keep/b"), "{ids:?}");
        assert!(!ids.contains("repo.zone/noise"), "zone itself not a member");
    }
}
