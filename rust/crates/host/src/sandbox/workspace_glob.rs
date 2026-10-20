use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// Normalize a workspace-relative glob or path: trim, strip `./`, reject `..`.
fn normalize_pattern(pattern: &str, label: &str) -> Result<String, String> {
    let pattern = pattern.trim().trim_start_matches("./");
    if pattern.is_empty() {
        return Err(format!("{label} must not be empty"));
    }
    if pattern.contains("..") {
        return Err(format!("{label} must not contain '..'"));
    }
    Ok(pattern.to_string())
}

/// Expand one positive include glob (no `!` handling).
fn expand_include(workspace_root: &Path, pattern: &str) -> Result<Vec<String>, String> {
    let pattern = normalize_pattern(pattern, "listFiles pattern")?;
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

/// Whether a workspace-relative `path` matches a glob `pattern` (`*`, `**`, `*.ext`).
///
/// No filesystem walk — used to apply `!` excludes after includes are expanded.
pub fn path_matches_glob(path: &str, pattern: &str) -> bool {
    let Ok(path) = normalize_pattern(path, "path") else {
        return false;
    };
    let Ok(pattern) = normalize_pattern(pattern, "pattern") else {
        return false;
    };
    let path_parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    let pat_parts: Vec<&str> = pattern.split('/').filter(|s| !s.is_empty()).collect();
    match_path_parts(&path_parts, &pat_parts)
}

fn match_path_parts(path: &[&str], pat: &[&str]) -> bool {
    if pat.is_empty() {
        return path.is_empty();
    }
    if pat[0] == "**" {
        if pat.len() == 1 {
            return true;
        }
        for i in 0..=path.len() {
            if match_path_parts(&path[i..], &pat[1..]) {
                return true;
            }
        }
        return false;
    }
    if path.is_empty() {
        return false;
    }
    if segment_matches_glob(path[0], pat[0]) {
        return match_path_parts(&path[1..], &pat[1..]);
    }
    false
}

fn segment_matches_glob(name: &str, pat: &str) -> bool {
    if pat == "*" {
        return true;
    }
    if pat.contains('*') {
        if pat.matches('*').count() != 1 {
            return false;
        }
        let Some((prefix, suffix)) = pat.split_once('*') else {
            return false;
        };
        return name.starts_with(prefix) && name.ends_with(suffix);
    }
    name == pat
}

/// Expand workspace globs with gitignore-style `!` excludes.
///
/// Positive patterns are unioned; then any path matching **any** `!` pattern is
/// dropped. Order does not matter (excludes win; no re-include).
pub fn resolve_globs(
    workspace_root: &Path,
    patterns: &[impl AsRef<str>],
) -> Result<Vec<String>, String> {
    let mut includes: Vec<String> = Vec::new();
    let mut excludes: Vec<String> = Vec::new();
    for raw in patterns {
        let raw = raw.as_ref().trim();
        if raw.is_empty() {
            return Err("glob pattern must not be empty".to_string());
        }
        if let Some(rest) = raw.strip_prefix('!') {
            let ex = normalize_pattern(rest, "exclude pattern after '!'")?;
            excludes.push(ex);
        } else {
            let inc = normalize_pattern(raw, "listFiles pattern")?;
            includes.push(inc);
        }
    }

    let mut universe: Vec<String> = Vec::new();
    for inc in &includes {
        universe.extend(expand_include(workspace_root, inc)?);
    }
    universe.sort();
    universe.dedup();

    if !excludes.is_empty() {
        universe.retain(|path| !excludes.iter().any(|ex| path_matches_glob(path, ex)));
    }
    Ok(universe)
}

/// Workspace-relative glob with a single `*` per path segment (e.g. `src/entities/*.dart`).
///
/// Thin wrapper around [`resolve_globs`] for a single pattern (supports a lone `!exclude`).
pub fn glob_workspace(workspace_root: &Path, pattern: &str) -> Result<Vec<String>, String> {
    resolve_globs(workspace_root, &[pattern])
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
        "**" => {
            // Zero directories: match the remainder at this level.
            if parts.len() == 1 {
                collect_files_recursive(base, out)?;
            } else {
                glob_segment(base, &parts[1..], out)?;
                let read_dir = match std::fs::read_dir(base) {
                    Ok(rd) => rd,
                    Err(_) => return Ok(()),
                };
                for entry in read_dir {
                    let entry = entry.map_err(|e| e.to_string())?;
                    if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
                        // Stay on `**` so nested directories are searched.
                        glob_segment(&entry.path(), parts, out)?;
                    }
                }
            }
        }
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

fn collect_files_recursive(base: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    let read_dir = match std::fs::read_dir(base) {
        Ok(rd) => rd,
        Err(_) => return Ok(()),
    };
    for entry in read_dir {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        if file_type.is_file() {
            out.push(entry.path());
        } else if file_type.is_dir() {
            collect_files_recursive(&entry.path(), out)?;
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
    fn glob_double_star_recursive() {
        let root = std::env::temp_dir().join(format!("codemod_glob_ds_{}", std::process::id()));
        let nested = root.join("lib/models/nested");
        fs::create_dir_all(&nested).unwrap();
        fs::write(root.join("lib/a.dart"), "").unwrap();
        fs::write(root.join("lib/models/b.dart"), "").unwrap();
        fs::write(nested.join("c.dart"), "").unwrap();
        fs::write(root.join("lib/models/skip.txt"), "").unwrap();
        let hits = glob_workspace(&root, "lib/**/*.dart").unwrap();
        assert_eq!(hits.len(), 3);
        assert!(hits.iter().any(|p| p.ends_with("a.dart")));
        assert!(hits.iter().any(|p| p.ends_with("b.dart")));
        assert!(hits.iter().any(|p| p.ends_with("c.dart")));
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn path_matches_glob_vendor_and_ext() {
        assert!(path_matches_glob("vendor/noise.rs", "**/vendor/**"));
        assert!(path_matches_glob("a/vendor/b/c.rs", "**/vendor/**"));
        assert!(!path_matches_glob("src/bin/tool.rs", "**/vendor/**"));
        assert!(path_matches_glob("lib/foo.dart", "lib/*.dart"));
        assert!(!path_matches_glob("lib/nested/foo.dart", "lib/*.dart"));
    }

    #[test]
    fn resolve_globs_exclude_vendor_keeps_src_bin() {
        let root = std::env::temp_dir().join(format!("codemod_resolve_{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("src/bin")).unwrap();
        fs::create_dir_all(root.join("vendor")).unwrap();
        fs::write(root.join("src/lib.rs"), "").unwrap();
        fs::write(root.join("src/bin/tool.rs"), "").unwrap();
        fs::write(root.join("vendor/noise.rs"), "").unwrap();

        let a = resolve_globs(&root, &["**/*.rs", "!**/vendor/**"]).unwrap();
        let b = resolve_globs(&root, &["!**/vendor/**", "**/*.rs"]).unwrap();
        assert_eq!(a, b, "order must not matter");
        assert!(a.iter().any(|p| p == "src/bin/tool.rs"));
        assert!(a.iter().any(|p| p == "src/lib.rs"));
        assert!(!a.iter().any(|p| p.contains("vendor")));

        // Specific include still lost to exclude (excludes win).
        let c = resolve_globs(&root, &["vendor/noise.rs", "!**/vendor/**"]).unwrap();
        assert!(c.is_empty());

        let alone = resolve_globs(&root, &["!**/vendor/**"]).unwrap();
        assert!(alone.is_empty());

        let _ = fs::remove_dir_all(root);
    }
}
