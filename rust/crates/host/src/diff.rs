use crate::protocol::{FilePreview, PatchInfo};
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
    let snippet = snippet_lines.and_then(|n| snippet_from_patches(patches, n));
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
}
