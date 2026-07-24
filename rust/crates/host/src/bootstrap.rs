use std::path::Path;

use serde_json::json;

include!(concat!(env!("OUT_DIR"), "/bootstrap_manifest.rs"));

/// Soft (default) vs cookin-style strict edit policy for the installed rule pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditPolicy {
    Recommend,
    Strict,
}

impl EditPolicy {
    pub fn as_str(self) -> &'static str {
        match self {
            EditPolicy::Recommend => "recommend",
            EditPolicy::Strict => "strict",
        }
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        match s {
            "recommend" => Ok(EditPolicy::Recommend),
            "strict" => Ok(EditPolicy::Strict),
            other => Err(format!(
                "unknown edit_policy {other:?}; expected \"recommend\" or \"strict\""
            )),
        }
    }
}

const KNOWN_COMPANIONS: &[&str] = &["codebase-memory"];

/// Validate companion names; returns sorted unique list or an error.
pub fn parse_companions(raw: &[String]) -> Result<Vec<String>, String> {
    let mut out = Vec::new();
    for name in raw {
        if !KNOWN_COMPANIONS.contains(&name.as_str()) {
            return Err(format!(
                "unknown companion {name:?}; known: {}",
                KNOWN_COMPANIONS.join(", ")
            ));
        }
        if !out.contains(name) {
            out.push(name.clone());
        }
    }
    out.sort();
    Ok(out)
}

/// Map an embedded export-relative path to a workspace dest path for the selected packs.
/// Returns `None` when the file belongs to a pack that is not selected.
fn resolve_dest_path(
    embed_path: &str,
    edit_policy: EditPolicy,
    companions: &[String],
) -> Option<String> {
    if let Some(rest) = embed_path.strip_prefix("rulesets/recommend/") {
        return if edit_policy == EditPolicy::Recommend {
            Some(rest.to_string())
        } else {
            None
        };
    }
    if let Some(rest) = embed_path.strip_prefix("rulesets/strict/") {
        return if edit_policy == EditPolicy::Strict {
            Some(rest.to_string())
        } else {
            None
        };
    }
    if let Some(rest) = embed_path.strip_prefix("companions/codebase-memory/") {
        return if companions.iter().any(|c| c == "codebase-memory") {
            Some(rest.to_string())
        } else {
            None
        };
    }
    // Unknown pack prefixes are never installed.
    if embed_path.starts_with("rulesets/") || embed_path.starts_with("companions/") {
        return None;
    }
    Some(embed_path.to_string())
}

pub fn bootstrap_project(
    workspace_root: &Path,
    force: bool,
    edit_policy: EditPolicy,
    companions: &[String],
) -> serde_json::Value {
    let mut written = Vec::new();
    let mut skipped = Vec::new();
    let mut errors = Vec::new();

    for file in EXPORTED_FILES {
        let Some(dest_rel) = resolve_dest_path(file.path, edit_policy, companions) else {
            continue;
        };
        let dest = workspace_root.join(&dest_rel);
        if dest.exists() && !force {
            skipped.push(dest_rel);
            continue;
        }

        if let Some(parent) = dest.parent() {
            if let Err(e) = std::fs::create_dir_all(parent) {
                errors.push(format!("{dest_rel}: failed to create parent dir: {e}"));
                continue;
            }
        }

        match std::fs::write(&dest, file.content) {
            Ok(()) => written.push(dest_rel),
            Err(e) => errors.push(format!("{dest_rel}: {e}")),
        }
    }

    let _ = std::fs::create_dir_all(workspace_root.join(".codemod/recipes"));
    let _ = std::fs::create_dir_all(workspace_root.join(".codemod/maps"));
    let _ = std::fs::create_dir_all(workspace_root.join(".codemod/variables"));

    let companions_json: Vec<&str> = companions.iter().map(String::as_str).collect();

    if errors.is_empty() {
        json!({
            "ok": true,
            "edit_policy": edit_policy.as_str(),
            "companions": companions_json,
            "written": written,
            "skipped": skipped,
        })
    } else {
        json!({
            "ok": false,
            "error": errors.join("; "),
            "edit_policy": edit_policy.as_str(),
            "companions": companions_json,
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

    fn temp_ws(label: &str) -> std::path::PathBuf {
        let ws = std::env::temp_dir().join(format!(
            "codemod_bootstrap_{label}_{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&ws);
        fs::create_dir_all(&ws).expect("create temp workspace");
        ws
    }

    #[test]
    fn manifest_includes_export_files() {
        assert!(
            exported_file_count() >= 6,
            "expected at least 6 exported files, got {}",
            exported_file_count()
        );
    }

    #[test]
    fn resolve_dest_path_selects_packs() {
        assert_eq!(
            resolve_dest_path(
                "rulesets/recommend/.cursor/rules/codemod-recipe.mdc",
                EditPolicy::Recommend,
                &[]
            )
            .as_deref(),
            Some(".cursor/rules/codemod-recipe.mdc")
        );
        assert_eq!(
            resolve_dest_path(
                "rulesets/strict/.cursor/rules/codemod-recipe.mdc",
                EditPolicy::Recommend,
                &[]
            ),
            None
        );
        assert_eq!(
            resolve_dest_path(
                "rulesets/strict/.cursor/rules/codemod-recipe.mdc",
                EditPolicy::Strict,
                &[]
            )
            .as_deref(),
            Some(".cursor/rules/codemod-recipe.mdc")
        );
        assert_eq!(
            resolve_dest_path(
                "companions/codebase-memory/.cursor/rules/codebase-memory.mdc",
                EditPolicy::Recommend,
                &[]
            ),
            None
        );
        assert_eq!(
            resolve_dest_path(
                "companions/codebase-memory/.cursor/rules/codebase-memory.mdc",
                EditPolicy::Recommend,
                &["codebase-memory".into()]
            )
            .as_deref(),
            Some(".cursor/rules/codebase-memory.mdc")
        );
        assert_eq!(
            resolve_dest_path(".agents/skills/codemod-overview/SKILL.md", EditPolicy::Strict, &[])
                .as_deref(),
            Some(".agents/skills/codemod-overview/SKILL.md")
        );
    }

    #[test]
    fn parse_companions_rejects_unknown() {
        assert!(parse_companions(&["codebase-memory".into()]).is_ok());
        assert!(parse_companions(&["nope".into()]).is_err());
        assert!(EditPolicy::parse("recommend").is_ok());
        assert!(EditPolicy::parse("strict").is_ok());
        assert!(EditPolicy::parse("loose").is_err());
    }

    #[test]
    fn bootstrap_default_writes_recommend_not_companion() {
        let ws = temp_ws("default");
        let result = bootstrap_project(&ws, false, EditPolicy::Recommend, &[]);
        assert_eq!(result["ok"], true);
        assert_eq!(result["edit_policy"], "recommend");
        assert_eq!(result["companions"], json!([]));
        assert!(ws.join(".cursor/rules/codemod-recipe.mdc").is_file());
        assert!(!ws.join(".cursor/rules/codebase-memory.mdc").exists());
        let rule = fs::read_to_string(ws.join(".cursor/rules/codemod-recipe.mdc")).unwrap();
        assert!(rule.contains("edit_policy=recommend"));
        assert!(!rule.contains("Edit policy"));
        let _ = fs::remove_dir_all(&ws);
    }

    #[test]
    fn bootstrap_strict_and_companion() {
        let ws = temp_ws("strict_comp");
        let companions = vec!["codebase-memory".into()];
        let result = bootstrap_project(&ws, false, EditPolicy::Strict, &companions);
        assert_eq!(result["ok"], true);
        assert_eq!(result["edit_policy"], "strict");
        assert_eq!(result["companions"], json!(["codebase-memory"]));
        let rule = fs::read_to_string(ws.join(".cursor/rules/codemod-recipe.mdc")).unwrap();
        assert!(rule.contains("edit_policy=strict"));
        assert!(rule.contains("## Edit policy"));
        assert!(ws.join(".cursor/rules/codebase-memory.mdc").is_file());
        let companion =
            fs::read_to_string(ws.join(".cursor/rules/codebase-memory.mdc")).unwrap();
        assert!(companion.contains("companions=codebase-memory"));
        assert!(!companion.contains("home-ikusa-workspace"));
        let _ = fs::remove_dir_all(&ws);
    }

    #[test]
    fn bootstrap_writes_skills_and_is_idempotent() {
        let ws = temp_ws("idempotent");

        let first = bootstrap_project(&ws, false, EditPolicy::Recommend, &[]);
        assert_eq!(first["ok"], true);
        let written = first["written"].as_array().expect("written array");
        assert!(!written.is_empty());
        assert!(ws
            .join(".agents/skills/codemod-overview/SKILL.md")
            .is_file());
        assert!(ws.join(".cursor/rules/codemod-recipe.mdc").is_file());

        let second = bootstrap_project(&ws, false, EditPolicy::Recommend, &[]);
        assert_eq!(second["ok"], true);
        let skipped = second["skipped"].as_array().expect("skipped array");
        assert_eq!(skipped.len(), written.len());

        let overview = ws.join(".agents/skills/codemod-overview/SKILL.md");
        let original = fs::read_to_string(&overview).expect("read overview");
        fs::write(&overview, "mutated").expect("mutate");

        let forced = bootstrap_project(&ws, true, EditPolicy::Recommend, &[]);
        assert_eq!(forced["ok"], true);
        let restored = fs::read_to_string(&overview).expect("read restored");
        assert_ne!(restored, "mutated");
        assert_eq!(restored, original);

        let _ = fs::remove_dir_all(&ws);
    }
}
