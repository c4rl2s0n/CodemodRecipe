//! Luau index/graph facades for relate and verify phases.

mod builders;
mod script;

pub use builders::{build_graph_facade, build_graph_facade_from_json, build_index_facade};
