//! Tree-sitter–based leading comment/trivia expansion for remove/replace spans.

use tree_sitter::Node;

fn is_comment_kind(kind: &str) -> bool {
    kind == "comment"
        || kind.ends_with("_comment")
        || kind.contains("comment")
        || kind == "line_comment"
        || kind == "block_comment"
}

fn gap_has_blank_line(source: &str, prev_end: usize, next_start: usize) -> bool {
    if prev_end >= next_start {
        return false;
    }
    let gap = &source[prev_end..next_start];
    gap.contains("\n\n") || gap.contains("\r\n\r\n")
}

/// Extend `start` backward over comment siblings immediately above `node`.
pub fn leading_trivia_start(source: &str, node: Node<'_>, start: usize) -> usize {
    let mut new_start = start;
    let mut cursor = node.prev_sibling();

    while let Some(sib) = cursor {
        if gap_has_blank_line(source, sib.end_byte(), new_start) {
            break;
        }
        if is_comment_kind(sib.kind()) {
            new_start = sib.start_byte();
            cursor = sib.prev_sibling();
            continue;
        }
        if !sib.is_named() {
            cursor = sib.prev_sibling();
            continue;
        }
        break;
    }

    new_start
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_line_detected_in_gap() {
        let source = "x\n\n  int y;";
        let gap_start = 1;
        let gap_end = source.find("int").unwrap();
        assert!(gap_has_blank_line(source, gap_start, gap_end));
    }

    #[test]
    fn extends_over_line_and_block_comment_siblings() {
        use crate::registry::{ensure_language_downloaded, LanguageRegistry};
        use tree_sitter::Parser;

        ensure_language_downloaded("dart");
        let mut registry = LanguageRegistry::new();
        let language = registry.get("dart").expect("dart").adapter_language();

        let source = "class C {\n  // line\n  /* block */\n  final int x = 0;\n}\n";
        let mut parser = Parser::new();
        parser.set_language(&language).unwrap();
        let tree = parser.parse(source, None).unwrap();
        let root = tree.root_node();
        let class = root.named_child(0).expect("class");
        let body = class.child_by_field_name("body").expect("body");
        let mut field_decl = None;
        let mut i = 0;
        while let Some(child) = body.named_child(i) {
            if child.kind() == "declaration" {
                field_decl = Some(child);
                break;
            }
            i += 1;
        }
        let field_decl = field_decl.expect("field declaration");

        let start = field_decl.start_byte();
        let expanded = leading_trivia_start(source, field_decl, start);
        let prefix = &source[expanded..start];
        assert!(prefix.contains("// line"));
        assert!(prefix.contains("/* block */"));
    }
}
