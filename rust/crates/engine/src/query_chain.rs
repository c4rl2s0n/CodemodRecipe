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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::{ensure_language_downloaded, LanguageRegistry};
    use tree_sitter::{Parser, Query, QueryCursor};
    use tree_sitter::StreamingIterator;

    #[test]
    fn match_root_is_outermost_common_ancestor() {
        ensure_language_downloaded("dart");
        let mut registry = LanguageRegistry::new();
        let engine = registry.get("dart").expect("dart engine");
        let language = engine.adapter_language();

        let source = "class Foo { void bar() {} }";
        let mut parser = Parser::new();
        parser.set_language(&language).unwrap();
        let tree = parser.parse(source, None).unwrap();

        let query_text = "(class_definition name: (identifier) @className) @class";
        let query = Query::new(&language, query_text).unwrap();
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source.as_bytes());
        let m = matches.next().expect("one match");
        let root = match_root_node(m).expect("match root");
        assert_eq!(root.kind(), "class_definition");
    }
}
