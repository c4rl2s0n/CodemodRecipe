//! Kind-level architecture roll-up from resolved edges.

use std::collections::BTreeMap;

use crate::component::relations::types::{ComponentEdge, FlatInstance, KindEdge};

/// Aggregate resolved edges into the kind-level architecture graph.
pub fn kind_graph(edges: &[ComponentEdge], flat: &[FlatInstance]) -> Vec<KindEdge> {
    let by_id: BTreeMap<&str, &FlatInstance> = flat.iter().map(|f| (f.id.as_str(), f)).collect();
    let mut counts: BTreeMap<(String, String, String, bool), usize> = BTreeMap::new();
    for edge in edges {
        let Some(from) = by_id.get(edge.from.as_str()) else {
            continue;
        };
        let (to, external) = match (&edge.to, &edge.symbol) {
            (Some(to_id), _) => match by_id.get(to_id.as_str()) {
                Some(target) => (target.component_id.clone(), false),
                None => continue,
            },
            (None, Some(symbol)) => (symbol.clone(), true),
            (None, None) => continue,
        };
        *counts
            .entry((from.component_id.clone(), edge.kind.clone(), to, external))
            .or_default() += 1;
    }
    counts
        .into_iter()
        .map(|((from, kind, to, external), count)| KindEdge {
            from,
            kind,
            to,
            count,
            external,
        })
        .collect()
}

