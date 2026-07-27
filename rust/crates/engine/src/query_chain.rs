//! Match-root helpers for chained queries.

use tree_sitter::{Node, QueryMatch};

fn node_contains(ancestor: Node<'_>, descendant: Node<'_>) -> bool {
    if ancestor.id() == descendant.id() {
        return true;
    }
    ancestor.start_byte() <= descendant.start_byte() && ancestor.end_byte() >= descendant.end_byte()
}

/// Outermost common ancestor of all captures in a query match (chain scope for next step).
pub fn match_root_node<'a, 'b: 'a>(m: &'a QueryMatch<'b, 'a>) -> Option<Node<'a>> {
    let mut iter = m.captures.iter().map(|c| c.node);
    let first = iter.next()?;
    let mut root = first;
    for node in iter {
        root = lowest_common_ancestor(root, node)?;
    }
    Some(root)
}

fn lowest_common_ancestor<'a>(mut left: Node<'a>, mut right: Node<'a>) -> Option<Node<'a>> {
    if node_contains(left, right) {
        return Some(left);
    }
    if node_contains(right, left) {
        return Some(right);
    }
    while left.parent().is_some() || right.parent().is_some() {
        if left.start_byte() > right.start_byte() {
            left = left.parent()?;
        } else if right.start_byte() > left.start_byte() {
            right = right.parent()?;
        } else {
            let lp = left.parent()?;
            let rp = right.parent()?;
            if lp.id() == rp.id() {
                return Some(lp);
            }
            left = lp;
            right = rp;
        }
    }
    None
}
