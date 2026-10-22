//! Component subsystem — Luau scan/runtime entry in host; domain types/facade in
//! [`codemod_recipe_components`] (D-025 F).

pub mod cache;
pub mod coverage;
pub mod delegate;
pub mod entry;
pub mod host_facade;
pub mod runtime;
pub mod scan;
pub mod snapshot;

pub use cache::ComponentScanCache;
pub use codemod_recipe_components::{
    graph_render, membership, relation_metrics, relations, types, ComponentEdge, ComponentHost,
    FakeComponentHost, KindEdge,
};
pub use coverage::{component_coverage, ComponentCoverageResponse};
pub use graph_render::{render_instance_graph_mermaid, render_kind_graph_mermaid};
pub use membership::{member_kinds, validate_membership_forest, MembershipForest};
pub use runtime::{
    invoke_hook, invoke_operation, invoke_scaffold, invoke_stack_operation, ComponentHookRequest,
    ComponentInvokeRequest, ComponentScaffoldRequest, StackInvokeRequest,
};
pub use scan::{
    component_incremental_scan, component_scan, component_scan_inner,
    ComponentIncrementalScanRequest,
};
pub use snapshot::{
    persist_scan_snapshot, DEFAULT_INSTANCE_GRAPH_REL, DEFAULT_KIND_GRAPH_REL, DEFAULT_SNAPSHOT_REL,
};
pub use types::*;
