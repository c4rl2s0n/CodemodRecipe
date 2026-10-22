//! Kind-graph Mermaid rendering.

use crate::relations::KindEdge;

use super::style::{mermaid_id, mermaid_label};

/// Kind-level architecture graph: one line per `(fromComponent, kind, toComponent)`
/// roll-up, edge label `kind ×count` (the `×1` is omitted). External symbol targets
/// get `(external)` in their label. Caller should omit member-layout kinds.
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
