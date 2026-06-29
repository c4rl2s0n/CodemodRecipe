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
    recipe: &str,
    args: &BTreeMap<String, String>,
    snapshot_paths: &[&Path],
) -> String {
    let snapshots: BTreeMap<String, FileSnapshot> = snapshot_paths
        .iter()
        .map(|path| (path.to_string_lossy().to_string(), file_snapshot(path)))
        .collect();

    let payload = serde_json::json!({
        "recipe": recipe,
        "args": args,
        "snapshots": snapshots,
    });
    let serialized = serde_json::to_string(&payload).unwrap_or_default();
    format!("{:x}", md5::compute(serialized))
}

pub fn validate_preview_token(
    recipe: &str,
    args: &BTreeMap<String, String>,
    provided: &str,
    snapshot_paths: &[&Path],
) -> Result<(), String> {
    if provided.is_empty() {
        return Err("Missing previewToken (run preview first)".to_string());
    }
    let expected = compute_preview_token(recipe, args, snapshot_paths);
    if provided != expected {
        return Err("Stale previewToken (files changed since preview; re-run preview)".to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn token_is_stable_for_same_inputs() {
        let file = std::env::temp_dir().join(format!(
            "codemod_preview_token_test_{}.dart",
            std::process::id()
        ));
        std::fs::write(&file, "class A {}").unwrap();

        let mut args = BTreeMap::new();
        args.insert("file".to_string(), "a.dart".to_string());

        let t1 = compute_preview_token("r1", &args, &[&file]);
        let t2 = compute_preview_token("r1", &args, &[&file]);
        assert_eq!(t1, t2);

        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn token_is_stable_for_arg_key_order() {
        let file = std::env::temp_dir().join(format!(
            "codemod_preview_token_order_{}.dart",
            std::process::id()
        ));
        std::fs::write(&file, "class A {}").unwrap();

        let mut args_ab = BTreeMap::new();
        args_ab.insert("a".to_string(), "1".to_string());
        args_ab.insert("b".to_string(), "2".to_string());

        let mut args_ba = BTreeMap::new();
        args_ba.insert("b".to_string(), "2".to_string());
        args_ba.insert("a".to_string(), "1".to_string());

        let t1 = compute_preview_token("recipe", &args_ab, &[&file]);
        let t2 = compute_preview_token("recipe", &args_ba, &[&file]);
        assert_eq!(t1, t2);

        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn token_changes_when_snapshot_content_changes() {
        let file = std::env::temp_dir().join(format!(
            "codemod_preview_token_mut_{}.dart",
            std::process::id()
        ));
        std::fs::write(&file, "class A {}").unwrap();
        let args = BTreeMap::new();

        let before = compute_preview_token("x", &args, &[&file]);
        thread::sleep(Duration::from_millis(10));
        std::fs::write(&file, "class B {} // longer").unwrap();
        let after = compute_preview_token("x", &args, &[&file]);

        assert_ne!(before, after);
        let _ = std::fs::remove_file(file);
    }

    #[test]
    fn validate_rejects_stale_token() {
        let file = std::env::temp_dir().join(format!(
            "codemod_preview_token_stale_{}.dart",
            std::process::id()
        ));
        std::fs::write(&file, "class A {}\n").unwrap();
        let args = BTreeMap::new();
        let token = compute_preview_token("x", &args, &[&file]);
        thread::sleep(Duration::from_millis(10));
        std::fs::write(&file, "class B {} // mutated\n").unwrap();
        let err = validate_preview_token("x", &args, &token, &[&file]).unwrap_err();
        assert!(err.contains("Stale previewToken"));
        let _ = std::fs::remove_file(file);
    }
}
