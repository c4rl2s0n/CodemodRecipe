//! Mermaid rendering for kind and instance relation graphs.

mod instance;
mod kind;
mod style;

pub use instance::render_instance_graph_mermaid;
pub use kind::render_kind_graph_mermaid;
