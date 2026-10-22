//! Query compile helpers — paren balance and clearer tree-sitter errors.

/// If `query_text` has unbalanced `()`, return an actionable message (D-012).
/// Ignores parentheses inside double-quoted strings.
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

/// Map a tree-sitter `Query::new` failure into a clearer message body for
/// [`crate::engine::EngineError::Query`] (which already prefixes the label).
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

/// Compile a tree-sitter query, preferring an unbalanced-paren diagnosis (D-012).
pub fn compile_query(
    language: &tree_sitter::Language,
    query_text: &str,
) -> Result<tree_sitter::Query, String> {
    if let Some(msg) = unbalanced_parens_message(query_text) {
        return Err(msg);
    }
    tree_sitter::Query::new(language, query_text)
        .map_err(|e| format_query_compile_error(query_text, &e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_unclosed_paren() {
        let msg = unbalanced_parens_message("((identifier) @x").unwrap();
        assert!(msg.contains("unbalanced"), "{msg}");
        assert!(msg.contains("1 unclosed"), "{msg}");
    }

    #[test]
    fn detects_extra_close() {
        let msg = unbalanced_parens_message("(identifier)) @x").unwrap();
        assert!(msg.contains("extra ')'"), "{msg}");
    }

    #[test]
    fn ignores_parens_in_strings() {
        assert!(unbalanced_parens_message(r#"(identifier) @x (#eq? @x "(")"#).is_none());
    }

    #[test]
    fn wraps_invalid_syntax() {
        let out = format_query_compile_error("(x)", "Query error at 0:0. Invalid syntax:\n");
        assert!(out.contains("Hint:"), "{out}");
        assert!(out.contains("Invalid syntax"), "{out}");
    }
}
