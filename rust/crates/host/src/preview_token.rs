use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileSnapshot {
    exists: bool,
    #[serde(rename = "modifiedMs")]
    modified_ms: u128,
    size: u64,
}

pub fn file_snapshot(path: &Path) -> FileSnapshot {
    let meta = std::fs::metadata(path).ok();
    FileSnapshot {
        exists: meta.is_some(),
        modified_ms: meta
            .as_ref()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_millis()),
        size: meta.as_ref().map(std::fs::Metadata::len).unwrap_or(0),
    }
}

pub fn compute_preview_token(
    recipe: Option<&str>,
    inline_recipe: Option<&serde_json::Value>,
    args: &BTreeMap<String, String>,
    snapshot_paths: &[&Path],
) -> String {
    let snapshots: BTreeMap<String, FileSnapshot> = snapshot_paths
        .iter()
        .map(|path| (path.to_string_lossy().to_string(), file_snapshot(path)))
        .collect();

    let payload = serde_json::json!({
        "recipe": recipe,
        "inlineRecipe": inline_recipe,
        "args": args,
        "snapshots": snapshots,
    });
    let serialized = serde_json::to_string(&payload).unwrap_or_default();
    format!("{:x}", md5::compute(serialized))
}

pub fn validate_preview_token(
    recipe: Option<&str>,
    inline_recipe: Option<&serde_json::Value>,
    args: &BTreeMap<String, String>,
    provided: &str,
    snapshot_paths: &[&Path],
) -> Result<(), String> {
    if provided.is_empty() {
        return Err("Missing previewToken (run preview first)".to_string());
    }
    let expected = compute_preview_token(recipe, inline_recipe, args, snapshot_paths);
    if provided != expected {
        return Err("Stale previewToken (files changed since preview; re-run preview)".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn token_is_stable_for_same_inputs() {
        let args = BTreeMap::from([("file".to_string(), "a.dart".to_string())]);
        let file = PathBuf::from("/tmp/example.dart");
        let a = compute_preview_token(Some("r"), None, &args, &[file.as_path()]);
        let b = compute_preview_token(Some("r"), None, &args, &[file.as_path()]);
        assert_eq!(a, b);
    }

    #[test]
    fn token_is_stable_for_arg_key_order() {
        let args1 = BTreeMap::from([
            ("a".to_string(), "1".to_string()),
            ("b".to_string(), "2".to_string()),
        ]);
        let args2 = BTreeMap::from([
            ("b".to_string(), "2".to_string()),
            ("a".to_string(), "1".to_string()),
        ]);
        let file = PathBuf::from("/tmp/example.dart");
        let a = compute_preview_token(Some("r"), None, &args1, &[file.as_path()]);
        let b = compute_preview_token(Some("r"), None, &args2, &[file.as_path()]);
        assert_eq!(a, b);
    }

    #[test]
    fn token_changes_when_snapshot_content_changes() {
        let dir = std::env::temp_dir().join(format!("preview_token_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.dart");
        std::fs::write(&file, "version-one").unwrap();
        let args = BTreeMap::new();
        let t1 = compute_preview_token(Some("r"), None, &args, &[file.as_path()]);
        std::fs::write(&file, "version-two-is-longer").unwrap();
        let t2 = compute_preview_token(Some("r"), None, &args, &[file.as_path()]);
        assert_ne!(t1, t2);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn validate_rejects_stale_token() {
        let dir = std::env::temp_dir().join(format!("preview_token_stale_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.dart");
        std::fs::write(&file, "version-one").unwrap();
        let args = BTreeMap::new();
        let token = compute_preview_token(Some("r"), None, &args, &[file.as_path()]);
        std::fs::write(&file, "version-two-is-longer").unwrap();
        let err = validate_preview_token(Some("r"), None, &args, &token, &[file.as_path()]).unwrap_err();
        assert!(err.contains("Stale"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn inline_recipe_affects_token() {
        let args = BTreeMap::new();
        let file = PathBuf::from("/tmp/example.dart");
        let inline = serde_json::json!({"id": "inline"});
        let a = compute_preview_token(None, Some(&inline), &args, &[file.as_path()]);
        let b = compute_preview_token(Some("registered"), None, &args, &[file.as_path()]);
        assert_ne!(a, b);
    }
}
