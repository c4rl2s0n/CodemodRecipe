//! Mermaid sanitization helpers.

pub(crate) fn mermaid_id(raw: &str) -> String {
    raw.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

/// Escape a label for a quoted mermaid string.
pub(crate) fn mermaid_label(raw: &str) -> String {
    raw.replace('"', "#quot;")
}
