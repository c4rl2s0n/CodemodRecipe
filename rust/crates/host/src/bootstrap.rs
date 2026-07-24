use std::path::Path;

use serde_json::json;

include!(concat!(env!("OUT_DIR"), "/bootstrap_manifest.rs"));

pub fn bootstrap_project(workspace_root: &Path, force: bool) -> serde_json::Value {
    let mut written = Vec::new();
    let mut skipped = Vec::new();
    let mut errors = Vec::new();

    for file in EXPORTED_FILES {
        let dest = workspace_root.join(file.path);
        if dest.exists() && !force {
            skipped.push(file.path.to_string());
            continue;
        }

        if let Some(parent) = dest.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                errors.push(format!("{}: failed to create parent dir: {e}", file.path));
                continue;
            }
        }

        match std::fs::write(&dest, file.content) {
            Ok(()) => written.push(file.path.to_string()),
            Err(e) => errors.push(format!("{}: {e}", file.path)),
        }
    }

    let _ = std::fs::create_dir_all(workspace_root.join(".codemod/recipes"));
    let _ = std::fs::create_dir_all(workspace_root.join(".codemod/maps"));
    let _ = std::fs::create_dir_all(workspace_root.join(".codemod/variables"));

    if errors.is_empty() {
        json!({
            "ok": true,
            "written": written,
            "skipped": skipped,
        })
    } else {
        json!({
            "ok": false,
            "error": errors.join("; "),
            "written": written,
            "skipped": skipped,
        })
    }
}

pub fn exported_file_count() -> usize {
    EXPORTED_FILES.len()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn manifest_includes_export_files() {
        assert!(
            exported_file_count() >= 6,
            "expected at least 6 exported files, got {}",
            exported_file_count()
        );
    }

    #[test]
    fn bootstrap_writes_skills_and_is_idempotent() {
        let ws =
            std::env::temp_dir().join(format!("codemod_bootstrap_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&ws);
        fs::create_dir_all(&ws).expect("create temp workspace");

        let first = bootstrap_project(&ws, false);
        assert_eq!(first["ok"], true);
        let written = first["written"].as_array().expect("written array");
        assert!(!written.is_empty());
        assert!(ws
            .join(".agents/skills/codemod-overview/SKILL.md")
            .is_file());
        assert!(ws.join(".cursor/rules/codemod-recipe.mdc").is_file());

        let second = bootstrap_project(&ws, false);
        assert_eq!(second["ok"], true);
        let skipped = second["skipped"].as_array().expect("skipped array");
        assert_eq!(skipped.len(), written.len());

        let overview = ws.join(".agents/skills/codemod-overview/SKILL.md");
        let original = fs::read_to_string(&overview).expect("read overview");
        fs::write(&overview, "mutated").expect("mutate");

        let forced = bootstrap_project(&ws, true);
        assert_eq!(forced["ok"], true);
        let restored = fs::read_to_string(&overview).expect("read restored");
        assert_ne!(restored, "mutated");
        assert_eq!(restored, original);

        let _ = fs::remove_dir_all(&ws);
    }
}
