//! Relation metrics response shapes.

use serde::Serialize;

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
    pub by_kind: std::collections::BTreeMap<String, usize>,
}
