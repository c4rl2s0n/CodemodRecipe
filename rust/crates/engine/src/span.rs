//! Span helpers for tree-sitter capture edits.

pub fn insert_offset_at_anchor_end(source: &str, _start: usize, end: usize, is_block: bool) -> usize {
    if end == 0 {
        return end;
    }
    let bytes = source.as_bytes();
    let before_close = end - 1;
    if is_block || bytes.get(before_close) == Some(&b'}') {
        return start_of_line(bytes, before_close);
    }
    end
}

/// Expand remove/replace span to include leading doc comments and trailing semicolon/newline.
pub fn expand_declaration_span(source: &str, start: usize, end: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut new_start = start_of_line(bytes, start);

    while new_start > 0 {
        let prev_line_end = new_start.saturating_sub(1);
        let line_start = start_of_line(bytes, prev_line_end);
        let line = source[line_start..new_start].trim();
        if line.is_empty() || line.starts_with("///") || line.starts_with("//") {
            new_start = line_start;
        } else {
            break;
        }
    }

    let mut new_end = end;
    if new_end < bytes.len() && bytes[new_end] == b';' {
        new_end += 1;
    }
    if new_end < bytes.len() && bytes[new_end] == b'\n' {
        new_end += 1;
    }

    (new_start, new_end)
}

fn start_of_line(bytes: &[u8], pos: usize) -> usize {
    if pos == 0 || pos > bytes.len() {
        return 0;
    }
    let mut i = pos;
    while i > 0 {
        if bytes[i - 1] == b'\n' {
            return i;
        }
        i -= 1;
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_before_closing_brace_of_class_body() {
        let source = "class A {\n  int x;\n}";
        let body_end = source.len();
        let offset = insert_offset_at_anchor_end(source, 0, body_end, false);
        assert_eq!(&source[offset..], "}");
    }

    #[test]
    fn expands_remove_to_doc_comment_and_trailing_newline() {
        let source = "class A {\n  /// doc\n  int x = 1;\n  int y;\n}";
        let decl_start = source.find("int x").unwrap();
        let decl_end = decl_start + source[decl_start..].find(';').unwrap() + 1;
        let (start, end) = expand_declaration_span(source, decl_start, decl_end);
        let removed = &source[start..end];
        assert!(removed.contains("/// doc"));
        assert!(removed.contains("int x = 1"));
        assert!(end <= source.find("int y").unwrap());
    }
}
