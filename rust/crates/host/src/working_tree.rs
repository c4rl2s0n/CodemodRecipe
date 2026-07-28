//! In-memory working tree for sequential recipe collect.
//!
//! Steps mutate staged content per path; disk is only read to seed entries.
//! Finalize emits at most one [`FileChange`] per path.

use codemod_recipe_core::file_change::{FileChange, IfExists, IfMissing};
use codemod_recipe_core::patch::SourcePatch;
use std::collections::BTreeMap;
use std::path::Path;

use crate::path_sandbox::PathSandbox;

#[derive(Debug, Clone)]
enum Entry {
    /// Did not exist on disk; content may be further edited in memory.
    New { content: String },
    /// Existed on disk at first touch; current may equal original.
    Existing { original: String, current: String },
    /// Scheduled for removal; original is for preview.
    Deleted { original: String },
}

#[derive(Debug, Default)]
pub struct WorkingTree {
    entries: BTreeMap<String, Entry>,
}

impl WorkingTree {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn create(
        &mut self,
        sandbox: &PathSandbox,
        relative: &str,
        content: String,
        if_exists: IfExists,
    ) -> Result<(), String> {
        let absolute = sandbox
            .resolve_workspace_relative(relative)
            .map_err(|e| e.message)?;
        let on_disk = absolute.exists();

        match self.entries.get(relative) {
            None => {
                if on_disk {
                    match if_exists {
                        IfExists::Fail => {
                            return Err(format!("File already exists: {relative}"));
                        }
                        IfExists::Skip => {
                            let original = read_file(&absolute, relative)?;
                            self.entries.insert(
                                relative.to_string(),
                                Entry::Existing {
                                    original: original.clone(),
                                    current: original,
                                },
                            );
                        }
                    }
                } else {
                    self.entries
                        .insert(relative.to_string(), Entry::New { content });
                }
                Ok(())
            }
            Some(Entry::New { .. }) => Err(format!(
                "Cannot create {relative}: already staged as a new file"
            )),
            Some(Entry::Existing { .. }) => match if_exists {
                IfExists::Fail => Err(format!("File already exists: {relative}")),
                IfExists::Skip => Ok(()),
            },
            Some(Entry::Deleted { .. }) => Err(format!(
                "Cannot create {relative}: path was deleted earlier in this recipe"
            )),
        }
    }

    /// Seed from disk if needed, then replace staged content via `transform`.
    pub fn apply_edit<F>(
        &mut self,
        sandbox: &PathSandbox,
        relative: &str,
        transform: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        match self.entries.get(relative) {
            Some(Entry::Deleted { .. }) => {
                return Err(format!(
                    "Cannot edit {relative}: path was deleted earlier in this recipe"
                ));
            }
            Some(Entry::New { content }) => {
                let next = transform(content)?;
                self.entries
                    .insert(relative.to_string(), Entry::New { content: next });
                return Ok(());
            }
            Some(Entry::Existing { original, current }) => {
                let next = transform(current)?;
                let original = original.clone();
                self.entries.insert(
                    relative.to_string(),
                    Entry::Existing {
                        original,
                        current: next,
                    },
                );
                return Ok(());
            }
            None => {}
        }

        // Absent: seed from disk (edit requires an existing file).
        let absolute = sandbox
            .resolve_workspace_relative(relative)
            .map_err(|e| e.message)?;
        if !absolute.exists() {
            return Err(format!("Failed to read {relative}: file not found"));
        }
        let original = read_file(&absolute, relative)?;
        let next = transform(&original)?;
        self.entries.insert(
            relative.to_string(),
            Entry::Existing {
                original,
                current: next,
            },
        );
        Ok(())
    }

    pub fn delete(
        &mut self,
        sandbox: &PathSandbox,
        relative: &str,
        if_missing: IfMissing,
    ) -> Result<(), String> {
        match self.entries.get(relative) {
            Some(Entry::New { .. }) => {
                self.entries.remove(relative);
                return Ok(());
            }
            Some(Entry::Existing { original, .. }) => {
                let original = original.clone();
                self.entries
                    .insert(relative.to_string(), Entry::Deleted { original });
                return Ok(());
            }
            Some(Entry::Deleted { .. }) => match if_missing {
                IfMissing::Fail => {
                    return Err(format!("File not found: {relative}"));
                }
                IfMissing::Skip => return Ok(()),
            },
            None => {}
        }

        let absolute = sandbox
            .resolve_workspace_relative(relative)
            .map_err(|e| e.message)?;
        if !absolute.exists() {
            return match if_missing {
                IfMissing::Fail => Err(format!("File not found: {relative}")),
                IfMissing::Skip => Ok(()),
            };
        }
        let original = read_file(&absolute, relative)?;
        self.entries
            .insert(relative.to_string(), Entry::Deleted { original });
        Ok(())
    }

    pub fn finalize(self) -> Vec<FileChange> {
        let mut changes = Vec::new();
        for (path, entry) in self.entries {
            match entry {
                Entry::New { content } => {
                    changes.push(FileChange::Create {
                        path,
                        content,
                        if_exists: IfExists::Fail,
                        skipped: false,
                    });
                }
                Entry::Deleted { original } => {
                    changes.push(FileChange::Delete {
                        path,
                        source: original,
                        if_missing: IfMissing::Fail,
                        skipped: false,
                    });
                }
                Entry::Existing { original, current } => {
                    if current == original {
                        continue;
                    }
                    let len = original.len();
                    changes.push(FileChange::Patch {
                        path,
                        source: original,
                        patches: vec![SourcePatch::new(0, len, current)],
                    });
                }
            }
        }
        changes
    }
}

fn read_file(absolute: &Path, relative: &str) -> Result<String, String> {
    std::fs::read_to_string(absolute).map_err(|e| format!("Failed to read {relative}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static COUNTER: AtomicUsize = AtomicUsize::new(0);

    fn temp_dir() -> std::path::PathBuf {
        let n = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "working_tree_{}_{n}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn create_then_edit_finalizes_as_create() {
        let root = temp_dir();
        let sandbox = PathSandbox::new(root.clone());
        let mut tree = WorkingTree::new();
        tree.create(
            &sandbox,
            "a.dart",
            "class A {}\n".into(),
            IfExists::Skip,
        )
        .unwrap();
        tree.apply_edit(&sandbox, "a.dart", |s| Ok(format!("{s}// edited\n")))
            .unwrap();
        let changes = tree.finalize();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            FileChange::Create { content, skipped, .. } => {
                assert!(!skipped);
                assert!(content.contains("// edited"));
            }
            other => panic!("expected Create, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn skip_create_then_edit_existing_finalizes_as_patch() {
        let root = temp_dir();
        std::fs::write(root.join("a.dart"), "class A {}\n").unwrap();
        let sandbox = PathSandbox::new(root.clone());
        let mut tree = WorkingTree::new();
        tree.create(
            &sandbox,
            "a.dart",
            "UNUSED".into(),
            IfExists::Skip,
        )
        .unwrap();
        tree.apply_edit(&sandbox, "a.dart", |_| Ok("class A { A(); }\n".into()))
            .unwrap();
        let changes = tree.finalize();
        assert_eq!(changes.len(), 1);
        match &changes[0] {
            FileChange::Patch { source, patches, .. } => {
                assert_eq!(source, "class A {}\n");
                assert_eq!(patches.len(), 1);
                assert_eq!(patches[0].replacement, "class A { A(); }\n");
            }
            other => panic!("expected Patch, got {other:?}"),
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn edit_then_create_errors() {
        let root = temp_dir();
        std::fs::write(root.join("a.dart"), "class A {}\n").unwrap();
        let sandbox = PathSandbox::new(root.clone());
        let mut tree = WorkingTree::new();
        tree.apply_edit(&sandbox, "a.dart", |s| Ok(s.to_string()))
            .unwrap();
        let err = tree
            .create(
                &sandbox,
                "a.dart",
                "x".into(),
                IfExists::Fail,
            )
            .unwrap_err();
        assert!(err.contains("already exists"), "{err}");
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn delete_new_removes_entry() {
        let root = temp_dir();
        let sandbox = PathSandbox::new(root.clone());
        let mut tree = WorkingTree::new();
        tree.create(
            &sandbox,
            "a.dart",
            "x".into(),
            IfExists::Fail,
        )
        .unwrap();
        tree.delete(&sandbox, "a.dart", IfMissing::Fail).unwrap();
        assert!(tree.finalize().is_empty());
        let _ = std::fs::remove_dir_all(root);
    }
}
