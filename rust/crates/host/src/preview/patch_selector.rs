use codemod_recipe_core::apply::atomic_apply::FileOperation;
use codemod_recipe_core::apply::file_change::FileChange;
use codemod_recipe_core::apply::patch::{apply_patches, PatchError, SourcePatch};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Default)]
pub struct FileSelection {
    pub include: bool,
}

pub fn parse_selection(value: &serde_json::Value) -> BTreeMap<String, FileSelection> {
    let mut result = BTreeMap::new();
    let Some(files) = value.get("files").and_then(|v| v.as_object()) else {
        return result;
    };
    for (path, entry) in files {
        let include = entry
            .get("include")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        result.insert(path.clone(), FileSelection { include });
    }
    result
}

pub fn apply_selection(
    path: &str,
    source: &str,
    patches: &[SourcePatch],
    selection: &BTreeMap<String, FileSelection>,
) -> Result<Option<String>, PatchError> {
    if matches!(selection.get(path), Some(s) if !s.include) {
        return Ok(None);
    }
    if patches.is_empty() {
        return Ok(None);
    }
    Ok(Some(apply_patches(source, patches)?))
}

pub struct AppliedChange {
    pub relative_path: String,
    pub operation: FileOperation,
}

pub fn apply_changes_with_selection(
    registry_resolve: impl Fn(&str) -> Result<PathBuf, String>,
    changes: &[FileChange],
    selection: &BTreeMap<String, FileSelection>,
) -> Result<Vec<AppliedChange>, String> {
    let mut applied = Vec::new();

    for change in changes {
        if change.is_skipped() {
            continue;
        }
        let path = change.path();
        if matches!(selection.get(path), Some(s) if !s.include) {
            continue;
        }

        let absolute = registry_resolve(path)?;

        match change {
            FileChange::Patch {
                source, patches, ..
            } => {
                let content = match apply_selection(path, source, patches, selection) {
                    Ok(Some(result)) => result,
                    Ok(None) => continue,
                    Err(error) => return Err(error.to_string()),
                };
                applied.push(AppliedChange {
                    relative_path: path.to_string(),
                    operation: FileOperation::Write {
                        path: absolute,
                        content,
                    },
                });
            }
            FileChange::Create { content, .. } => {
                applied.push(AppliedChange {
                    relative_path: path.to_string(),
                    operation: FileOperation::Write {
                        path: absolute,
                        content: content.clone(),
                    },
                });
            }
            FileChange::Delete { .. } => {
                applied.push(AppliedChange {
                    relative_path: path.to_string(),
                    operation: FileOperation::Delete { path: absolute },
                });
            }
        }
    }

    Ok(applied)
}

#[cfg(test)]
mod tests {
    use super::*;
    use codemod_recipe_core::apply::patch::SourcePatch;

    #[test]
    fn drops_file_when_include_is_false() {
        let patches = vec![SourcePatch::new(0, 1, "A")];
        let mut selection = BTreeMap::new();
        selection.insert(
            "a.dart".to_string(),
            FileSelection { include: false },
        );
        let result = apply_selection("a.dart", "ab", &patches, &selection).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn applies_all_patches_when_file_included() {
        let patches = vec![SourcePatch::new(0, 1, "X"), SourcePatch::new(2, 3, "Y")];
        let result = apply_selection("a.dart", "abc", &patches, &BTreeMap::new()).unwrap();
        assert_eq!(result.as_deref(), Some("XbY"));
    }
}
