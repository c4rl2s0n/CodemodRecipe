use crate::patch::{apply_patches, validate_non_overlapping, PatchError, SourcePatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IfExists {
    #[default]
    Fail,
    Skip,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IfMissing {
    #[default]
    Fail,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChange {
    Patch {
        path: String,
        source: String,
        patches: Vec<SourcePatch>,
    },
    Create {
        path: String,
        content: String,
        if_exists: IfExists,
        format: bool,
        skipped: bool,
    },
    Delete {
        path: String,
        source: String,
        if_missing: IfMissing,
        skipped: bool,
    },
}

impl FileChange {
    pub fn path(&self) -> &str {
        match self {
            Self::Patch { path, .. } | Self::Create { path, .. } | Self::Delete { path, .. } => path,
        }
    }

    pub fn is_skipped(&self) -> bool {
        match self {
            Self::Patch { source, patches, .. } => {
                patches.is_empty()
                    || apply_patches(source, patches).ok() == Some(source.clone())
            }
            Self::Create { skipped, .. } | Self::Delete { skipped, .. } => *skipped,
        }
    }

    pub fn modified_content(&self) -> Result<Option<String>, PatchError> {
        match self {
            Self::Patch { source, patches, .. } => {
                if patches.is_empty() {
                    return Ok(None);
                }
                Ok(Some(apply_patches(source, patches)?))
            }
            Self::Create {
                content, skipped, ..
            } => {
                if *skipped {
                    Ok(None)
                } else {
                    Ok(Some(content.clone()))
                }
            }
            Self::Delete { skipped, .. } => {
                if *skipped {
                    Ok(None)
                } else {
                    Ok(Some(String::new()))
                }
            }
        }
    }

    pub fn original_content(&self) -> &str {
        match self {
            Self::Patch { source, .. } | Self::Delete { source, .. } => source,
            Self::Create { .. } => "",
        }
    }

    pub fn patches(&self) -> Option<&[SourcePatch]> {
        match self {
            Self::Patch { patches, .. } => Some(patches),
            _ => None,
        }
    }
}

/// Merge file changes, combining patches per path and rejecting incompatible mixes.
pub fn merge_file_changes(changes: Vec<FileChange>) -> Result<Vec<FileChange>, String> {
    let mut merged: Vec<FileChange> = Vec::new();

    for change in changes {
        let path = change.path().to_string();
        let existing_idx = merged.iter().position(|c| c.path() == path);

        match change {
            FileChange::Patch {
                path,
                source,
                mut patches,
            } => {
                if let Some(idx) = existing_idx {
                    match &mut merged[idx] {
                        FileChange::Patch {
                            source: existing_source,
                            patches: existing,
                            ..
                        } => {
                            if existing_source != &source {
                                return Err(format!("Conflicting source for patch file {path}"));
                            }
                            existing.append(&mut patches);
                            validate_non_overlapping(existing, existing_source.len())
                                .map_err(|e| e.to_string())?;
                        }
                        _ => {
                            return Err(format!(
                                "Cannot combine patch and full-file changes for {path}"
                            ));
                        }
                    }
                } else {
                    validate_non_overlapping(&patches, source.len()).map_err(|e| e.to_string())?;
                    merged.push(FileChange::Patch {
                        path,
                        source,
                        patches,
                    });
                }
            }
            FileChange::Create { .. } | FileChange::Delete { .. } => {
                if existing_idx.is_some() {
                    return Err(format!("Multiple full-file changes for {path}"));
                }
                merged.push(change);
            }
        }
    }

    Ok(merged)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::patch::SourcePatch;

    #[test]
    fn merges_patches_for_same_path() {
        let changes = vec![
            FileChange::Patch {
                path: "a.dart".to_string(),
                source: "ab".to_string(),
                patches: vec![SourcePatch::new(0, 0, "X")],
            },
            FileChange::Patch {
                path: "a.dart".to_string(),
                source: "ab".to_string(),
                patches: vec![SourcePatch::new(2, 2, "Y")],
            },
        ];
        let merged = merge_file_changes(changes).unwrap();
        assert_eq!(merged.len(), 1);
        match &merged[0] {
            FileChange::Patch { patches, .. } => assert_eq!(patches.len(), 2),
            _ => panic!("expected patch"),
        }
    }

    #[test]
    fn rejects_patch_and_create_on_same_path() {
        let changes = vec![
            FileChange::Patch {
                path: "a.dart".to_string(),
                source: "ab".to_string(),
                patches: vec![SourcePatch::new(0, 0, "X")],
            },
            FileChange::Create {
                path: "a.dart".to_string(),
                content: "new".to_string(),
                if_exists: IfExists::Fail,
                format: false,
                skipped: false,
            },
        ];
        assert!(merge_file_changes(changes).is_err());
    }
}
