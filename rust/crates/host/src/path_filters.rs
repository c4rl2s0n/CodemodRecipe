//! Pure string path transforms for MiniJinja filters (`parent`, `basename`, `stem`).
//!
//! No workspace I/O — preview/validate stay deterministic.

use std::path::Path;

/// Normalize separators to `/` and strip trailing slashes.
fn normalize(input: &str) -> String {
    let s = input.replace('\\', "/");
    s.trim_end_matches('/').to_string()
}

fn path_as_slash_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// Parent directory of `input`, or empty if none.
pub fn path_parent(input: &str) -> String {
    let normalized = normalize(input);
    if normalized.is_empty() {
        return String::new();
    }
    Path::new(&normalized)
        .parent()
        .map(path_as_slash_string)
        .unwrap_or_default()
}

/// Final path component of `input`, or empty if none.
pub fn path_basename(input: &str) -> String {
    let normalized = normalize(input);
    if normalized.is_empty() {
        return String::new();
    }
    Path::new(&normalized)
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Filename without extension; for names with no `.`, same as basename.
pub fn path_stem(input: &str) -> String {
    let normalized = normalize(input);
    if normalized.is_empty() {
        return String::new();
    }
    Path::new(&normalized)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_of_nested_path() {
        assert_eq!(
            path_parent("lib/features/feed/widgets"),
            "lib/features/feed"
        );
        assert_eq!(path_parent("lib/foo.dart"), "lib");
    }

    #[test]
    fn parent_strips_trailing_slash() {
        assert_eq!(
            path_parent("lib/features/feed/widgets/"),
            "lib/features/feed"
        );
    }

    #[test]
    fn parent_of_bare_name_is_empty() {
        assert_eq!(path_parent("foo.dart"), "");
        assert_eq!(path_parent("widgets"), "");
    }

    #[test]
    fn parent_of_empty_is_empty() {
        assert_eq!(path_parent(""), "");
        assert_eq!(path_parent("/"), "");
        assert_eq!(path_parent("///"), "");
    }

    #[test]
    fn parent_normalizes_backslashes() {
        assert_eq!(path_parent(r"lib\features\feed"), "lib/features");
    }

    #[test]
    fn basename_final_component() {
        assert_eq!(path_basename("lib/features/feed/widgets"), "widgets");
        assert_eq!(path_basename("lib/foo.dart"), "foo.dart");
        assert_eq!(path_basename("widgets"), "widgets");
    }

    #[test]
    fn basename_strips_trailing_slash() {
        assert_eq!(path_basename("lib/features/feed/widgets/"), "widgets");
    }

    #[test]
    fn basename_of_empty_is_empty() {
        assert_eq!(path_basename(""), "");
    }

    #[test]
    fn basename_normalizes_backslashes() {
        assert_eq!(path_basename(r"lib\features\feed"), "feed");
    }

    #[test]
    fn stem_strips_extension() {
        assert_eq!(path_stem("lib/foo.dart"), "foo");
        assert_eq!(path_stem("foo.dart"), "foo");
        assert_eq!(path_stem("archive.tar.gz"), "archive.tar");
    }

    #[test]
    fn stem_without_extension_matches_basename() {
        assert_eq!(path_stem("lib/features/feed/widgets"), "widgets");
        assert_eq!(path_stem("widgets"), "widgets");
    }

    #[test]
    fn stem_of_empty_is_empty() {
        assert_eq!(path_stem(""), "");
    }

    #[test]
    fn chain_parent_then_basename() {
        let dir = "lib/features/feed/widgets";
        assert_eq!(path_basename(&path_parent(dir)), "feed");
    }
}
