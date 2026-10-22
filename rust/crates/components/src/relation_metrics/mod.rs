//! Relation connectivity metrics for scan snapshots.

mod compute;
mod types;

pub use compute::{ignore_coverage_instance_ids, relation_metrics};
pub use types::{RelationAggregate, RelationByComponent, RelationMetrics, RelationRatios};
