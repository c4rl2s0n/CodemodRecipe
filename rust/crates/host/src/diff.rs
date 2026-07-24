use crate::protocol::{FilePreview, PatchInfo};
use codemod_recipe_core::file_change::FileChange;
use codemod_recipe_core::patch::SourcePatch;

pub fn patches_to_patch_info(patches: &[SourcePatch], include_replacement: bool) -> Vec<PatchInfo> {
    patches
        .iter()
        .enumerate()
        .map(|(index, patch)| {
            let length = patch.end.saturating_sub(patch.start);
            let preview = if include_replacement {
                Some(preview_replacement(&patch.replacement))
            } else {
                None
            };
            PatchInfo {
                index,
                offset: patch.start,
                length,
                replacement: if include_replacement {
                    Some(patch.replacement.clone())
                } else {
                    None
                },
                replacement_preview: preview,
                description: patch.description.clone(),
            }
        })
        .collect()
}

pub fn snippet_from_patches(patches: &[SourcePatch], max_lines: u32) -> Option<String> {
    let first = patches.first()?;
    if first.replacement.is_empty() {
        return None;
    }
    let normalized = first.replacement.replace("\r\n", "\n");
    let lines: Vec<_> = normalized.lines().take(max_lines as usize).collect();
    let snippet = lines.join("\n").trim_end().to_string();
    if snippet.is_empty() {
        None
    } else {
        Some(snippet)
    }
}

/// Prefer a window around the first differing line when a patch replaces the whole file.
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
        // Only deletions / trailing changes — show end of modified or empty.
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

fn is_full_span_replace(source: &str, patches: &[SourcePatch]) -> bool {
    matches!(
        patches,
        [p] if p.start == 0 && p.end == source.len() && !p.replacement.is_empty()
    )
}

pub fn build_file_preview(
    path: String,
    original: &str,
    modified: &str,
    patches: &[SourcePatch],
    include_contents: bool,
    include_replacements: bool,
    snippet_lines: Option<u32>,
) -> FilePreview {
    let skipped = original == modified;
    let snippet = snippet_lines.and_then(|n| {
        if is_full_span_replace(original, patches) {
            snippet_from_original_and_modified(original, modified, n)
        } else {
            snippet_from_patches(patches, n)
        }
    });
    FilePreview {
        path,
        kind: "edit",
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
        patches: patches_to_patch_info(patches, include_replacements),
        snippet,
    }
}

pub fn build_file_preview_from_change(
    change: &FileChange,
    include_contents: bool,
    include_replacements: bool,
    snippet_lines: Option<u32>,
) -> Result<FilePreview, codemod_recipe_core::patch::PatchError> {
    let path = change.path().to_string();
    match change {
        FileChange::Patch {
            source, patches, ..
        } => {
            let modified = if patches.is_empty() {
                source.clone()
            } else {
                codemod_recipe_core::patch::apply_patches(source, patches)?
            };
            Ok(build_file_preview(
                path,
                source,
                &modified,
                patches,
                include_contents,
                include_replacements,
                snippet_lines,
            ))
        }
        FileChange::Create {
            content, skipped, ..
        } => Ok(FilePreview {
            path,
            kind: "create",
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
            patches: vec![],
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
            kind: "delete",
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
            patches: vec![],
            snippet: None,
        }),
    }
}

fn preview_replacement(replacement: &str) -> String {
    const MAX: usize = 120;
    if replacement.len() <= MAX {
        return replacement.to_string();
    }
    format!("{}...", &replacement[..MAX])
}

#[cfg(test)]
mod tests {
    use super::*;
    use codemod_recipe_core::file_change::{FileChange, IfExists};
    use codemod_recipe_core::patch::SourcePatch;

    #[test]
    fn serializes_patch_offset_and_length() {
        let patches = vec![SourcePatch::new(10, 15, "hello")];
        let info = patches_to_patch_info(&patches, true);
        assert_eq!(info[0].offset, 10);
        assert_eq!(info[0].length, 5);
        assert_eq!(info[0].replacement.as_deref(), Some("hello"));
    }

    #[test]
    fn snippet_from_first_patch_replacement() {
        let patches = vec![SourcePatch::new(10, 10, "line one\nline two\nline three")];
        let preview = build_file_preview(
            "a.dart".to_string(),
            "class A {}",
            "class A { x }",
            &patches,
            false,
            false,
            Some(2),
        );
        assert_eq!(preview.snippet.as_deref(), Some("line one\nline two"));
    }

    #[test]
    fn snippet_for_full_span_replace_starts_at_first_diff() {
        let original = "line1\nline2\nline3\n";
        let modified = "line1\nCHANGED\nline3\n";
        let patches = vec![SourcePatch::new(0, original.len(), modified)];
        let preview = build_file_preview(
            "a.dart".to_string(),
            original,
            modified,
            &patches,
            false,
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
            format: true,
            skipped: false,
        };
        let preview = build_file_preview_from_change(&change, true, false, None).unwrap();
        assert_eq!(preview.kind, "create");
        assert!(preview.is_new);
        assert_eq!(preview.modified.as_deref(), Some("class New {}\n"));
    }
}
