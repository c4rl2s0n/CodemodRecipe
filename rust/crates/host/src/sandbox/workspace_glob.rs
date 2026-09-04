use std::path::{Path, PathBuf};

/// Workspace-relative glob with a single `*` per path segment (e.g. `src/entities/*.dart`).
pub fn glob_workspace(workspace_root: &Path, pattern: &str) -> Result<Vec<String>, String> {
    let pattern = pattern.trim().trim_start_matches("./");
    if pattern.is_empty() {
        return Err("glob pattern must not be empty".to_string());
    }
    if pattern.contains("..") {
        return Err("glob pattern must not contain '..'".to_string());
    }
    let parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let mut matches = Vec::new();
    glob_segment(workspace_root, &parts, &mut matches)?;
    let mut relative: Vec<String> = matches
        .iter()
        .filter_map(|p| {
            p.strip_prefix(workspace_root)
                .ok()
                .map(|r| r.to_string_lossy().replace('\\', "/"))
        })
        .collect();
    relative.sort();
    relative.dedup();
    Ok(relative)
}

fn glob_segment(base: &Path, parts: &[&str], out: &mut Vec<PathBuf>) -> Result<(), String> {
    if parts.is_empty() {
        if base.is_file() {
            out.push(base.to_path_buf());
        }
        return Ok(());
    }
    match parts[0] {
        "*" if parts.len() == 1 => {
            let read_dir = std::fs::read_dir(base).map_err(|e| e.to_string())?;
            for entry in read_dir {
                let entry = entry.map_err(|e| e.to_string())?;
                if entry.file_type().map_err(|e| e.to_string())?.is_file() {
                    out.push(entry.path());
                }
            }
        }
        "*" => {
            let read_dir = std::fs::read_dir(base).map_err(|e| e.to_string())?;
            for entry in read_dir {
                let entry = entry.map_err(|e| e.to_string())?;
                if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                    glob_segment(&entry.path(), &parts[1..], out)?;
                }
            }
        }
        segment if segment.contains('*') => {
            let (prefix, suffix) = match segment.split_once('*') {
                Some(parts) => parts,
                None => {
                    return Err(format!(
                        "only one wildcard per segment is supported, got {segment}"
                    ));
                }
            };
            if segment.matches('*').count() > 1 {
                return Err(format!(
                    "only one wildcard per segment is supported, got {segment}"
                ));
            }
            let read_dir = std::fs::read_dir(base).map_err(|e| e.to_string())?;
            for entry in read_dir {
                let entry = entry.map_err(|e| e.to_string())?;
                let file_type = entry.file_type().map_err(|e| e.to_string())?;
                let name = entry.file_name();
                let name = name.to_string_lossy();
                let matches = name.starts_with(prefix) && name.ends_with(suffix);
                if parts.len() == 1 {
                    if file_type.is_file() && matches {
                        out.push(entry.path());
                    }
                } else if file_type.is_dir() && matches {
                    glob_segment(&entry.path(), &parts[1..], out)?;
                }
            }
        }
        segment => {
            let next = base.join(segment);
            if next.exists() {
                glob_segment(&next, &parts[1..], out)?;
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn glob_finds_files_in_segment() {
        let root = std::env::temp_dir().join(format!("codemod_glob_{}", std::process::id()));
        let entities = root.join("src/entities");
        fs::create_dir_all(&entities).unwrap();
        fs::write(entities.join("foo.dart"), "class Foo {}").unwrap();
        fs::write(entities.join("bar.dart"), "class Bar {}").unwrap();
        let hits = glob_workspace(&root, "src/entities/*.dart").unwrap();
        assert_eq!(hits.len(), 2);
        assert!(hits.iter().any(|p| p.ends_with("foo.dart")));
        let _ = fs::remove_dir_all(root);
    }
}
