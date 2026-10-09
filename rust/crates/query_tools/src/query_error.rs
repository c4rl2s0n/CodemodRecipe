//! Clearer tree-sitter query compile errors (D-012).

/// If `query_text` has unbalanced `()`, return an actionable message.
pub fn unbalanced_parens_message(query_text: &str) -> Option<String> {
    let bytes = query_text.as_bytes();
    let mut depth = 0i32;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            b';' if i + 1 < bytes.len() && bytes[i + 1] == b';' => {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth < 0 {
                    return Some(
                        "unbalanced parentheses (extra ')'). \
                         Check captures, grouping, and that every '(' has a matching ')'."
                            .into(),
                    );
                }
            }
            _ => {}
        }
        i += 1;
    }
    if depth > 0 {
        return Some(format!(
            "unbalanced parentheses ({depth} unclosed '('). \
             Check captures, grouping, and that every '(' has a matching ')'."
        ));
    }
    None
}

pub fn format_query_compile_error(query_text: &str, ts_error: &str) -> String {
    if let Some(msg) = unbalanced_parens_message(query_text) {
        return msg;
    }
    if ts_error.contains("Invalid syntax") {
        return format!(
            "{ts_error}\n\
             Hint: check capture placement (`(type @cap …)` preferred), balanced parentheses, \
             and that predicates `(#…)` come after captures within a pattern."
        );
    }
    ts_error.to_string()
}
