use crate::protocol::DiagnosticSource;

/// Build a diagnostic source, locating the first occurrence of `needle` in `text`
/// when provided (1-based line/column). When `text` is `None`, attempts to read `file`.
pub fn source_with_needle(file: &str, text: Option<&str>, needle: &str) -> DiagnosticSource {
    if needle.is_empty() {
        return source_file_only(file);
    }
    let owned;
    let content = match text {
        Some(c) => c,
        None => match std::fs::read_to_string(file) {
            Ok(c) => {
                owned = c;
                owned.as_str()
            }
            Err(_) => return source_file_only(file),
        },
    };
    if let Some((line, column)) = find_line_column(content, needle) {
        return DiagnosticSource {
            file: file.to_string(),
            line: Some(line),
            column: Some(column),
        };
    }
    source_file_only(file)
}

/// Like [`source_with_needle`], but reads `file` from disk relative to `workspace_root`
/// when `text` is not already available.
pub fn source_with_needle_in_workspace(
    workspace_root: &std::path::Path,
    file: &str,
    text: Option<&str>,
    needle: &str,
) -> DiagnosticSource {
    if let Some(content) = text {
        return source_with_needle(file, Some(content), needle);
    }
    let path = if std::path::Path::new(file).is_absolute() {
        std::path::PathBuf::from(file)
    } else {
        workspace_root.join(file)
    };
    match std::fs::read_to_string(path) {
        Ok(content) => source_with_needle(file, Some(&content), needle),
        Err(_) => source_file_only(file),
    }
}

pub fn source_file_only(file: &str) -> DiagnosticSource {
    DiagnosticSource {
        file: file.to_string(),
        line: None,
        column: None,
    }
}

/// Find 1-based line and column of the first occurrence of `needle`.
pub fn find_line_column(text: &str, needle: &str) -> Option<(u32, u32)> {
    let idx = text.find(needle)?;
    let before = &text[..idx];
    let line = before.bytes().filter(|&b| b == b'\n').count() as u32 + 1;
    let column = match before.rfind('\n') {
        Some(n) => (before.len() - n) as u32,
        None => before.len() as u32 + 1,
    };
    Some((line, column))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_line_and_column() {
        let text = "id: foo\ngroup: rust.data\nsteps: []\n";
        assert_eq!(find_line_column(text, "group:"), Some((2, 1)));
        assert_eq!(find_line_column(text, "rust.data"), Some((2, 8)));
    }

    #[test]
    fn source_with_needle_sets_spans() {
        let text = "id: demo\n";
        let src = source_with_needle(".codemod/recipes/a.yaml", Some(text), "id: demo");
        assert_eq!(src.line, Some(1));
        assert_eq!(src.column, Some(1));
    }
}
