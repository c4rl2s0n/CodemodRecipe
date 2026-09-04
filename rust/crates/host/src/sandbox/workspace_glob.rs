use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Workspace-relative glob with a single `*` per path segment (e.g. `src/entities/*.dart`).
pub fn glob_workspace(workspace_root: &Path, pattern: &str) -> Result<Vec<String>, String> {
    let pattern = pattern.trim().trim_start_matches("./");
    if pattern.is_empty() {
        return Err("listFiles pattern must not be empty".to_string());
    }
    if pattern.contains("..") {
        return Err("listFiles pattern must not contain '..'".to_string());
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

/// Match workspace files against a `{placeholder}` path pattern.
/// Each `{name}` captures one contiguous run of characters within a single path segment.
/// Returns rows with `path` plus named capture fields.
pub fn match_workspace(
    workspace_root: &Path,
    pattern: &str,
) -> Result<Vec<BTreeMap<String, String>>, String> {
    let pattern = pattern.trim().trim_start_matches("./");
    if pattern.is_empty() {
        return Err("matchFiles pattern must not be empty".to_string());
    }
    if pattern.contains("..") {
        return Err("matchFiles pattern must not contain '..'".to_string());
    }
    if pattern.contains('*') {
        return Err(
            "matchFiles uses {placeholder} patterns; use listFiles for * globs".to_string(),
        );
    }
    let parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    let mut matches = Vec::new();
    match_segment(workspace_root, &parts, BTreeMap::new(), &mut matches)?;
    let mut relative: Vec<BTreeMap<String, String>> = matches
        .into_iter()
        .filter_map(|(path, mut captures)| {
            let rel = path
                .strip_prefix(workspace_root)
                .ok()?
                .to_string_lossy()
                .replace('\\', "/");
            captures.insert("path".to_string(), rel);
            Some(captures)
        })
        .collect();
    relative.sort_by(|a, b| a.get("path").cmp(&b.get("path")));
    Ok(relative)
}

fn match_segment(
    base: &Path,
    parts: &[&str],
    captures: BTreeMap<String, String>,
    out: &mut Vec<(PathBuf, BTreeMap<String, String>)>,
) -> Result<(), String> {
    if parts.is_empty() {
        if base.is_file() {
            out.push((base.to_path_buf(), captures));
        }
        return Ok(());
    }
    let segment = parts[0];
    if segment.contains('{') {
        let read_dir = std::fs::read_dir(base).map_err(|e| e.to_string())?;
        for entry in read_dir {
            let entry = entry.map_err(|e| e.to_string())?;
            let file_type = entry.file_type().map_err(|e| e.to_string())?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            let Some(seg_caps) = match_placeholders(segment, &name) else {
                continue;
            };
            let mut merged = captures.clone();
            for (k, v) in seg_caps {
                if k == "path" {
                    return Err("matchFiles placeholder name 'path' is reserved".to_string());
                }
                merged.insert(k, v);
            }
            if parts.len() == 1 {
                if file_type.is_file() {
                    out.push((entry.path(), merged));
                }
            } else if file_type.is_dir() {
                match_segment(&entry.path(), &parts[1..], merged, out)?;
            }
        }
    } else {
        let next = base.join(segment);
        if next.exists() {
            match_segment(&next, &parts[1..], captures, out)?;
        }
    }
    Ok(())
}

/// Match `text` against a segment pattern containing `{name}` placeholders.
pub fn match_placeholders(pattern: &str, text: &str) -> Option<BTreeMap<String, String>> {
    let mut captures = BTreeMap::new();
    let mut pi = 0;
    let mut ti = 0;
    let pb = pattern.as_bytes();
    while pi < pb.len() {
        if pb[pi] == b'{' {
            let close = pattern[pi + 1..].find('}')? + pi + 1;
            let key = &pattern[pi + 1..close];
            if key.is_empty() {
                return None;
            }
            let after = &pattern[close + 1..];
            let next_ph = after.find('{').unwrap_or(after.len());
            let lit = &after[..next_ph];
            let capture_end = if lit.is_empty() {
                text.len()
            } else {
                text[ti..].find(lit)? + ti
            };
            let value = &text[ti..capture_end];
            if value.is_empty() {
                return None;
            }
            captures.insert(key.to_string(), value.to_string());
            ti = capture_end;
            pi = close + 1;
        } else {
            let next = pattern[pi..]
                .find('{')
                .map(|i| pi + i)
                .unwrap_or(pattern.len());
            let lit = &pattern[pi..next];
            if !text[ti..].starts_with(lit) {
                return None;
            }
            ti += lit.len();
            pi = next;
        }
    }
    if ti != text.len() {
        return None;
    }
    Some(captures)
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

    #[test]
    fn match_files_captures_placeholder() {
        let root = std::env::temp_dir().join(format!("codemod_match_{}", std::process::id()));
        let stores = root.join("lib/stores");
        fs::create_dir_all(&stores).unwrap();
        fs::write(stores.join("recipe_store.dart"), "").unwrap();
        fs::write(stores.join("user_store.dart"), "").unwrap();
        let rows = match_workspace(&root, "lib/stores/{id}_store.dart").unwrap();
        assert_eq!(rows.len(), 2);
        let recipe = rows
            .iter()
            .find(|r| r["path"].ends_with("recipe_store.dart"))
            .unwrap();
        assert_eq!(recipe.get("id").map(String::as_str), Some("recipe"));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn match_placeholders_mid_segment() {
        let caps = match_placeholders("{id}_store.dart", "recipe_store.dart").unwrap();
        assert_eq!(caps.get("id").map(String::as_str), Some("recipe"));
    }
}
