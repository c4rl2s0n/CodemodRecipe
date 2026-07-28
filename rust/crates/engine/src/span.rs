//! Span helpers for tree-sitter capture edits.

pub fn insert_offset_at_anchor_end(
    source: &str,
    _start: usize,
    end: usize,
    is_block: bool,
) -> usize {
    if end == 0 {
        return end;
    }
    let bytes = source.as_bytes();
    let before_close = end - 1;
    if is_block || bytes.get(before_close) == Some(&b'}') {
        return line_start(bytes, before_close);
    }
    end
}

/// Expand remove/replace span to include leading doc comments and trailing semicolon/newline.
pub fn expand_declaration_span(source: &str, start: usize, end: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut new_start = line_start(bytes, start);

    while new_start > 0 {
        let prev_line_end = new_start.saturating_sub(1);
        let line_start_pos = line_start(bytes, prev_line_end);
        let line = source[line_start_pos..new_start].trim();
        if line.is_empty() || line.starts_with("///") || line.starts_with("//") {
            new_start = line_start_pos;
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

/// Expand remove/replace span for C-style languages (Java, Kotlin, Rust).
pub fn expand_cstyle_declaration_span(source: &str, start: usize, end: usize) -> (usize, usize) {
    let bytes = source.as_bytes();
    let mut new_start = line_start(bytes, start);

    while new_start > 0 {
        let prev_line_end = new_start.saturating_sub(1);
        let line_start_pos = line_start(bytes, prev_line_end);
        let line = source[line_start_pos..new_start].trim();
        if line.is_empty() || line.starts_with("//") {
            new_start = line_start_pos;
        } else {
            break;
        }
    }

    while new_start > 0 {
        let block_start = find_block_comment_start(bytes, new_start);
        if let Some(block_start) = block_start {
            new_start = block_start;
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

/// After `end`, skip whitespace and consume a trailing statement `;` and following newline.
pub fn expand_trailing_semicolon(source: &str, end: usize) -> usize {
    let bytes = source.as_bytes();
    let mut new_end = end;
    while new_end < bytes.len() && bytes[new_end].is_ascii_whitespace() && bytes[new_end] != b'\n' {
        new_end += 1;
    }
    // Eat trailing `;` (declarations) or `,` (parameter / argument lists).
    if new_end < bytes.len() && (bytes[new_end] == b';' || bytes[new_end] == b',') {
        new_end += 1;
    }
    if new_end < bytes.len() && bytes[new_end] == b'\n' {
        new_end += 1;
    }
    new_end
}

fn find_block_comment_start(bytes: &[u8], pos: usize) -> Option<usize> {
    if pos < 2 {
        return None;
    }
    let search_end = line_start(bytes, pos.saturating_sub(1));
    let window = &bytes[search_end..pos];
    window
        .windows(2)
        .rposition(|w| w == b"/*")
        .map(|idx| search_end + idx)
}

pub fn line_start_offset(source: &str, pos: usize) -> usize {
    line_start(source.as_bytes(), pos)
}

fn line_start(bytes: &[u8], pos: usize) -> usize {
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

    #[test]
    fn expands_cstyle_block_comment() {
        let source = "class A {\n  /** doc */\n  int x = 1;\n  int y;\n}";
        let decl_start = source.find("int x").unwrap();
        let decl_end = decl_start + source[decl_start..].find(';').unwrap() + 1;
        let (start, end) = expand_cstyle_declaration_span(source, decl_start, decl_end);
        let removed = &source[start..end];
        assert!(removed.contains("/** doc */"));
        assert!(removed.contains("int x = 1"));
    }
}
