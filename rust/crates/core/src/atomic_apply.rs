use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FileWrite {
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone)]
pub enum FileOperation {
    Write { path: PathBuf, content: String },
    Delete { path: PathBuf },
}

/// Applies file writes atomically: backup originals, commit all, rollback on failure.
pub fn apply_files_atomically(writes: &[FileWrite]) -> Result<(), String> {
    let ops: Vec<FileOperation> = writes
        .iter()
        .map(|w| FileOperation::Write {
            path: w.path.clone(),
            content: w.content.clone(),
        })
        .collect();
    apply_operations_atomically(&ops)
}

/// Applies mixed write/delete operations atomically with rollback on failure.
pub fn apply_operations_atomically(ops: &[FileOperation]) -> Result<(), String> {
    if ops.is_empty() {
        return Ok(());
    }

    let mut backups: Vec<Backup> = Vec::with_capacity(ops.len());

    for op in ops {
        let path = match op {
            FileOperation::Write { path, .. } | FileOperation::Delete { path } => path.clone(),
        };
        let original = fs::read_to_string(&path).ok();
        backups.push(Backup { path, original });
    }

    for op in ops {
        match op {
            FileOperation::Write { path, content } => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .map_err(|e| format!("Failed to create {}: {e}", parent.display()))?;
                }
                if let Err(error) = fs::write(path, content) {
                    rollback(&backups);
                    return Err(format!("Failed to write {}: {error}", path.display()));
                }
            }
            FileOperation::Delete { path } => {
                if path.exists() {
                    if let Err(error) = fs::remove_file(path) {
                        rollback(&backups);
                        return Err(format!("Failed to delete {}: {error}", path.display()));
                    }
                }
            }
        }
    }

    Ok(())
}

fn rollback(backups: &[Backup]) {
    for backup in backups {
        match &backup.original {
            Some(content) => {
                let _ = fs::write(&backup.path, content);
            }
            None => {
                let _ = fs::remove_file(&backup.path);
            }
        }
    }
}

struct Backup {
    path: PathBuf,
    original: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(prefix: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{prefix}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ))
    }

    #[test]
    fn rolls_back_all_files_when_a_later_write_fails() {
        let dir = temp_dir("atomic_apply_rollback");
        fs::create_dir_all(&dir).unwrap();

        let ok_path = dir.join("ok.dart");
        let bad_path = dir.join("bad.dart");
        fs::write(&ok_path, "original ok\n").unwrap();
        fs::write(&bad_path, "original bad\n").unwrap();

        let _ = fs::remove_file(&bad_path);
        fs::create_dir_all(&bad_path).unwrap();

        let writes = vec![
            FileWrite {
                path: ok_path.clone(),
                content: "patched ok\n".to_string(),
            },
            FileWrite {
                path: bad_path.clone(),
                content: "patched bad\n".to_string(),
            },
        ];

        assert!(apply_files_atomically(&writes).is_err());
        assert_eq!(fs::read_to_string(&ok_path).unwrap(), "original ok\n");

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn commits_all_files_when_every_change_succeeds() {
        let dir = temp_dir("atomic_apply_ok");
        fs::create_dir_all(&dir).unwrap();

        let file = dir.join("counter.dart");
        fs::write(&file, "class Counter {}\n").unwrap();

        apply_files_atomically(&[FileWrite {
            path: file.clone(),
            content: "class Counter { int value = 0; }\n".to_string(),
        }])
        .unwrap();

        assert_eq!(
            fs::read_to_string(&file).unwrap(),
            "class Counter { int value = 0; }\n"
        );

        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn deletes_file_and_rolls_back_on_failure() {
        let dir = temp_dir("atomic_delete");
        fs::create_dir_all(&dir).unwrap();

        let keep = dir.join("keep.dart");
        let remove = dir.join("remove.dart");
        fs::write(&keep, "keep\n").unwrap();
        fs::write(&remove, "remove\n").unwrap();

        let bad = dir.join("bad.dart");
        fs::create_dir_all(&bad).unwrap();

        let ops = vec![
            FileOperation::Delete {
                path: remove.clone(),
            },
            FileOperation::Write {
                path: keep.clone(),
                content: "patched\n".to_string(),
            },
            FileOperation::Write {
                path: bad.clone(),
                content: "fail\n".to_string(),
            },
        ];

        assert!(apply_operations_atomically(&ops).is_err());
        assert_eq!(fs::read_to_string(&keep).unwrap(), "keep\n");
        assert_eq!(fs::read_to_string(&remove).unwrap(), "remove\n");

        let _ = fs::remove_dir_all(dir);
    }
}
