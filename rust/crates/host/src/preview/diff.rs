use crate::protocol::FilePreview;
use codemod_recipe_core::apply::file_change::FileChange;
use codemod_recipe_yaml::preview_kinds;

/// Prefer a window around the first differing line when comparing two file texts.
pub fn snippet_from_original_and_modified(
    original: &str,
    modified: &str,
    max_lines: u32,
) -> Option<String> {
    if max_lines == 0 || original == modified {
        return None;
    }
    let orig_norm = original.replace("\r\n", "\n");
    let mod_norm = modified.replace("\r\n", "\n");
    let orig_lines: Vec<_> = orig_norm.lines().collect();
    let mod_lines: Vec<_> = mod_norm.lines().collect();
    let mut first_diff = 0usize;
    let limit = orig_lines.len().min(mod_lines.len());
    while first_diff < limit && orig_lines[first_diff] == mod_lines[first_diff] {
        first_diff += 1;
    }
    if first_diff >= mod_lines.len() {
        if mod_lines.is_empty() {
            return None;
        }
        first_diff = mod_lines.len().saturating_sub(max_lines as usize);
    }
    let snippet = mod_lines
        .iter()
        .skip(first_diff)
        .take(max_lines as usize)
        .copied()
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string();
    if snippet.is_empty() {
        None
    } else {
        Some(snippet)
    }
}

pub fn build_file_preview(
    path: String,
    original: &str,
    modified: &str,
    include_contents: bool,
    snippet_lines: Option<u32>,
) -> FilePreview {
    let skipped = original == modified;
    let snippet =
        snippet_lines.and_then(|n| snippet_from_original_and_modified(original, modified, n));
    FilePreview {
        path,
        kind: preview_kinds::EDIT,
        is_new: false,
        skipped,
        original: if include_contents {
            Some(original.to_string())
        } else {
            None
        },
        modified: if include_contents {
            Some(modified.to_string())
        } else {
            None
        },
        snippet,
    }
}

pub fn build_file_preview_from_change(
    change: &FileChange,
    include_contents: bool,
    snippet_lines: Option<u32>,
) -> Result<FilePreview, codemod_recipe_core::apply::patch::PatchError> {
    let path = change.path().to_string();
    match change {
        FileChange::Patch {
            source, patches, ..
        } => {
            let modified = if patches.is_empty() {
                source.clone()
            } else {
                codemod_recipe_core::apply::patch::apply_patches(source, patches)?
            };
            Ok(build_file_preview(
                path,
                source,
                &modified,
                include_contents,
                snippet_lines,
            ))
        }
        FileChange::Create {
            content, skipped, ..
        } => Ok(FilePreview {
            path,
            kind: preview_kinds::CREATE,
            is_new: true,
            skipped: *skipped,
            original: if include_contents {
                Some(String::new())
            } else {
                None
            },
            modified: if include_contents {
                Some(content.clone())
            } else {
                None
            },
            snippet: snippet_lines.and_then(|n| {
                let lines: Vec<_> = content.lines().take(n as usize).collect();
                let snippet = lines.join("\n");
                if snippet.is_empty() {
                    None
                } else {
                    Some(snippet)
                }
            }),
        }),
        FileChange::Delete {
            source, skipped, ..
        } => Ok(FilePreview {
            path,
            kind: preview_kinds::DELETE,
            is_new: false,
            skipped: *skipped,
            original: if include_contents {
                Some(source.clone())
            } else {
                None
            },
            modified: if include_contents {
                Some(String::new())
            } else {
                None
            },
            snippet: None,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codemod_recipe_core::apply::file_change::{FileChange, IfExists};
    use codemod_recipe_core::apply::patch::SourcePatch;

    #[test]
    fn snippet_starts_at_first_diff() {
        let original = "line1\nline2\nline3\n";
        let modified = "line1\nCHANGED\nline3\n";
        let preview = build_file_preview(
            "a.dart".to_string(),
            original,
            modified,
            false,
            Some(2),
        );
        assert_eq!(preview.snippet.as_deref(), Some("CHANGED\nline3"));
    }

    #[test]
    fn previews_create_file() {
        let change = FileChange::Create {
            path: "lib/new.dart".to_string(),
            content: "class New {}\n".to_string(),
            if_exists: IfExists::Fail,
            skipped: false,
        };
        let preview = build_file_preview_from_change(&change, true, None).unwrap();
        assert_eq!(preview.kind, "create");
        assert!(preview.is_new);
        assert_eq!(preview.modified.as_deref(), Some("class New {}\n"));
    }

    #[test]
    fn patch_change_builds_preview_from_original_and_modified() {
        let change = FileChange::Patch {
            path: "a.dart".to_string(),
            source: "hello world".to_string(),
            patches: vec![SourcePatch::new(6, 11, "Rust")],
        };
        let preview = build_file_preview_from_change(&change, true, None).unwrap();
        assert_eq!(preview.original.as_deref(), Some("hello world"));
        assert_eq!(preview.modified.as_deref(), Some("hello Rust"));
    }
}
