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
}
