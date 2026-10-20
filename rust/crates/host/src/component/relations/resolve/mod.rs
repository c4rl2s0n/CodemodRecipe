//! Normalize and resolve relate-hook edges; kind-level roll-up.

mod edges;
mod findings;
mod kind;
mod normalize;
mod targets;

pub use edges::resolve_edges;
pub use kind::kind_graph;
pub use normalize::{normalize_raw_edges, RawEdge};
