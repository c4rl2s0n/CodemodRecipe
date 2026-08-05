//! Inject synthetic captures on nested named patterns for depth visualization.

use crate::QueryToolsError;

#[derive(Debug, Clone)]
pub struct InstrumentedQuery {
    pub query: String,
    pub layer_ranges: Vec<LayerRange>,
}

#[derive(Debug, Clone)]
pub struct LayerRange {
    pub name: String,
    pub depth: u32,
    pub query_start: usize,
    pub query_end: usize,
}

/// Instrument a tree-sitter query by adding `@__layer_N` captures on named patterns
/// that do not already have a capture. Predicates `(#...)` are copied as-is.
///
/// Tree-sitter allows both `(node) @cap` and `(node @cap)`; both count as captured.
pub fn instrument_query(source: &str) -> Result<InstrumentedQuery, QueryToolsError> {
    let mut layer_ranges = Vec::new();
    let mut layer_idx = 0u32;
    let out = instrument_slice(
        source.as_bytes(),
        source,
        0,
        source.len(),
        0,
        &mut layer_ranges,
        &mut layer_idx,
    )?;
    Ok(InstrumentedQuery {
        query: out,
        layer_ranges,
    })
}

fn instrument_slice(
    bytes: &[u8],
    source: &str,
    start: usize,
    end: usize,
    depth: u32,
    layers: &mut Vec<LayerRange>,
    layer_idx: &mut u32,
) -> Result<String, QueryToolsError> {
    let mut out = String::new();
    let mut i = start;
    while i < end {
        if bytes[i] == b';' && i + 1 < end && bytes[i + 1] == b';' {
            let s = i;
            while i < end && bytes[i] != b'\n' {
                i += 1;
            }
            out.push_str(&source[s..i]);
            continue;
        }

        if bytes[i] == b'(' {
            let after = skip_ws(bytes, i + 1, end);
            if after < end && bytes[after] == b'#' {
                let close_end = skip_balanced(bytes, i, end)?;
                out.push_str(&source[i..close_end]);
                i = close_end;
                continue;
            }
            if after < end && is_ident_start(bytes[after]) {
                let kind_end = scan_ident(bytes, after, end);
                let close_end = skip_balanced(bytes, i, end)?;
                let close = close_end - 1;
                let pattern_depth = depth + 1;
                let body = instrument_slice(
                    bytes,
                    source,
                    kind_end,
                    close,
                    pattern_depth,
                    layers,
                    layer_idx,
                )?;
                let rebuilt = format!("({}{}", &source[after..kind_end], body);

                // Capture may be inside `(node @c)` or after `(node) @c`
                let (ext_cap_end, has_ext) = peek_external_capture(bytes, close_end, end);
                let has_inner = trailing_capture_body(&body);
                // Do not inject @__layer_* into patterns that use tree-sitter `.`
                // anchors — placing a capture after `.` is invalid syntax.
                let has_dot_anchor = body_has_dot_anchor(&body);

                if has_inner || has_ext || has_dot_anchor {
                    out.push_str(&rebuilt);
                    out.push(')');
                    if has_ext {
                        out.push_str(&source[close_end..ext_cap_end]);
                        i = ext_cap_end;
                    } else {
                        i = close_end;
                    }
                } else {
                    let name = format!("__layer_{layer_idx}");
                    layers.push(LayerRange {
                        name: name.clone(),
                        depth: pattern_depth,
                        query_start: i,
                        query_end: close_end,
                    });
                    *layer_idx += 1;
                    out.push_str(&rebuilt);
                    out.push(' ');
                    out.push('@');
                    out.push_str(&name);
                    out.push(')');
                    i = close_end;
                }
                continue;
            }
        }

        let ch = source[i..end].chars().next().unwrap_or('\0');
        out.push(ch);
        i += ch.len_utf8();
    }
    Ok(out)
}

/// If `@ident` follows `from`, return (end_index, true).
fn peek_external_capture(bytes: &[u8], from: usize, end: usize) -> (usize, bool) {
    let after = skip_ws(bytes, from, end);
    if after >= end || bytes[after] != b'@' {
        return (from, false);
    }
    if after + 1 >= end || !is_ident_start(bytes[after + 1]) {
        return (from, false);
    }
    let name_end = scan_ident(bytes, after + 1, end);
    (name_end, true)
}

fn trailing_capture_body(body: &str) -> bool {
    let trimmed = body.trim_end();
    let bytes = trimmed.as_bytes();
    let mut j = bytes.len();
    while j > 0 && bytes[j - 1].is_ascii_whitespace() {
        j -= 1;
    }
    let end = j;
    while j > 0 && (bytes[j - 1].is_ascii_alphanumeric() || bytes[j - 1] == b'_') {
        j -= 1;
    }
    j > 0 && bytes[j - 1] == b'@' && end > j
}

/// True if `body` contains a tree-sitter `.` anchor (not part of an identifier).
fn body_has_dot_anchor(body: &str) -> bool {
    let bytes = body.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                if bytes[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
            continue;
        }
        if bytes[i] == b'.' {
            let prev_ok = i == 0
                || (!bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_');
            let next_ok = i + 1 >= bytes.len()
                || (!bytes[i + 1].is_ascii_alphanumeric() && bytes[i + 1] != b'_');
            if prev_ok && next_ok {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn skip_balanced(bytes: &[u8], start: usize, end: usize) -> Result<usize, QueryToolsError> {
    if start >= end || bytes[start] != b'(' {
        return Err(QueryToolsError::Query("expected '('".into()));
    }
    let mut depth = 0i32;
    let mut i = start;
    while i < end {
        match bytes[i] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i + 1);
                }
            }
            b'"' => {
                i += 1;
                while i < end && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(QueryToolsError::Query("unbalanced parentheses".into()))
}

fn skip_ws(bytes: &[u8], mut i: usize, end: usize) -> usize {
    while i < end && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn scan_ident(bytes: &[u8], start: usize, end: usize) -> usize {
    let mut i = start;
    while i < end && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
        i += 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instruments_nested_patterns() {
        let q = "(annotation name: (identifier) @n (arguments))";
        let inst = instrument_query(q).unwrap();
        assert!(inst.query.contains("@n"), "{}", inst.query);
        assert!(inst.query.contains("@__layer_"), "{}", inst.query);
        assert!(
            inst.query.contains("(arguments) @__layer_")
                || inst.query.contains("(arguments @__layer_"),
            "{}",
            inst.query
        );
    }

    #[test]
    fn skips_predicates() {
        let q = r#"(identifier) @n (#eq? @n "Foo")"#;
        let inst = instrument_query(q).unwrap();
        assert!(inst.query.contains(r#"(#eq? @n "Foo")"#), "{}", inst.query);
        assert!(inst.query.contains("(identifier) @n"), "{}", inst.query);
    }

    #[test]
    fn does_not_double_capture() {
        let q = "(identifier) @n";
        let inst = instrument_query(q).unwrap();
        assert_eq!(inst.query.trim(), "(identifier) @n");
        assert!(inst.layer_ranges.is_empty());
    }

    #[test]
    fn skips_layer_inject_when_dot_anchor_present() {
        let q = "(list_literal (identifier) @target .)";
        let inst = instrument_query(q).unwrap();
        assert!(
            !inst.query.contains("@__layer_"),
            "must not inject layer after .: {}",
            inst.query
        );
        assert!(inst.query.contains("@target ."), "{}", inst.query);
        // Round-trip: instrumented query must still parse as valid S-expr shape
        assert!(inst.query.contains("(list_literal"), "{}", inst.query);
    }

    #[test]
    fn skips_layer_on_sibling_dot() {
        let q = "(pair (identifier) @a . (identifier) @b)";
        let inst = instrument_query(q).unwrap();
        assert!(
            !inst.query.contains("@__layer_"),
            "{}",
            inst.query
        );
    }

    #[test]
    fn still_layers_parents_without_dot() {
        let q = "(class_definition (annotation (arguments)))";
        let inst = instrument_query(q).unwrap();
        assert!(inst.query.contains("@__layer_"), "{}", inst.query);
    }
}
