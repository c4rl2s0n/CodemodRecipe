use crate::protocol::DiagnosticSource;
use codemod_recipe_core::resource_path::resolve_under_root;
use std::path::{Path, PathBuf};

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
/// when `text` is not already available, using the shared exact-path resolver for
/// relative paths under the workspace.
pub fn source_with_needle_in_workspace(
    workspace_root: &Path,
    file: &str,
    text: Option<&str>,
    needle: &str,
) -> DiagnosticSource {
    if let Some(content) = text {
        return source_with_needle(file, Some(content), needle);
    }
    let path = if Path::new(file).is_absolute() {
        PathBuf::from(file)
    } else {
        match resolve_under_root(workspace_root, file) {
            Ok(path) => path,
            Err(_) => return source_file_only(file),
        }
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

/// Build a source from an explicit 1-based line/column.
pub fn source_at(file: &str, line: u32, column: u32) -> DiagnosticSource {
    DiagnosticSource {
        file: file.to_string(),
        line: Some(line),
        column: Some(column),
    }
}

/// Prefer serde_yaml's location when present; otherwise fall back to file-only.
pub fn source_from_serde_yaml_error(file: &str, err: &serde_yaml::Error) -> DiagnosticSource {
    if let Some(loc) = err.location() {
        return source_at(file, loc.line() as u32, loc.column() as u32);
    }
    source_file_only(file)
}

/// Locate a YAML syntax failure for `text` by re-running serde_yaml so we can
/// recover line/column even when the original error was stringified.
pub fn source_from_yaml_text(file: &str, text: &str) -> DiagnosticSource {
    match serde_yaml::from_str::<serde_yaml::Value>(text) {
        Err(e) => source_from_serde_yaml_error(file, &e),
        Ok(_) => source_file_only(file),
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
        let text = "id: foo\nsteps: []\n";
        assert_eq!(find_line_column(text, "steps:"), Some((2, 1)));
        assert_eq!(find_line_column(text, "[]"), Some((2, 8)));
    }

    #[test]
    fn source_with_needle_sets_spans() {
        let text = "id: demo\n";
        let src = source_with_needle(".codemod/recipes/a.yaml", Some(text), "id: demo");
        assert_eq!(src.line, Some(1));
        assert_eq!(src.column, Some(1));
    }

    #[test]
    fn source_from_serde_yaml_error_has_location() {
        let err = serde_yaml::from_str::<serde_yaml::Value>("@bad").expect_err("invalid yaml");
        let src = source_from_serde_yaml_error("bad.yaml", &err);
        assert_eq!(src.line, Some(1));
        assert!(src.column.unwrap_or(0) >= 1);
    }

    #[test]
    fn source_with_needle_in_workspace_rejects_traversal() {
        let workspace = std::env::temp_dir().join(format!(
            "codemod_diag_source_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&workspace).unwrap();
        std::fs::write(workspace.join("recipe.yaml"), "id: demo\n").unwrap();

        let src = source_with_needle_in_workspace(&workspace, "../outside.yaml", None, "id:");
        assert_eq!(src.line, None);
        assert_eq!(src.column, None);

        let _ = std::fs::remove_dir_all(workspace);
    }
}
