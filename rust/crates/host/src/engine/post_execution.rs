use codemod_recipe_yaml::model::PostExecution;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::template::render_template;
use codemod_recipe_core::resource_path::resolve_existing_resource;

/// Run post-apply actions after a successful apply.
///
/// Each string entry is Jinja-rendered with recipe args/maps/vars. If the rendered
/// value resolves to an existing file under the codemod root, the file body is
/// Jinja-rendered and executed via bash. Otherwise the rendered string is run
/// with `sh -c` (cwd = workspace root). No builtins and no injected path lists.
pub fn run_post_execution(
    actions: &[PostExecution],
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    workspace_root: &Path,
    codemod_root: &Path,
    recipe_file: Option<&Path>,
) -> Result<(), String> {
    for action in actions {
        match action {
            PostExecution::String(entry) => {
                let rendered = render_template(entry, args, maps, vars)?;
                let trimmed = rendered.trim();
                if trimmed.is_empty() {
                    return Err("postExecution entry rendered to an empty string".to_string());
                }
                if let Some(script_path) =
                    resolve_existing_script(workspace_root, codemod_root, recipe_file, trimmed)?
                {
                    run_script_file(&script_path, args, maps, vars, workspace_root)?;
                } else {
                    run_shell_command(trimmed, workspace_root)?;
                }
            }
            PostExecution::Map(_) => {
                return Err(
                    "postExecution map/object entries are not supported; use a string command or a script path relative to the codemod root"
                        .to_string(),
                );
            }
        }
    }
    Ok(())
}

fn resolve_existing_script(
    workspace_root: &Path,
    codemod_root: &Path,
    recipe_file: Option<&Path>,
    rendered: &str,
) -> Result<Option<PathBuf>, String> {
    // Shell commands typically contain whitespace; skip script resolution for those.
    if rendered.chars().any(char::is_whitespace) {
        return Ok(None);
    }
    let resolved = resolve_existing_resource(rendered, recipe_file, codemod_root, None)
        .map_err(|e| e.message)?;
    if let Some(path) = resolved {
        let root = workspace_root
            .canonicalize()
            .unwrap_or_else(|_| workspace_root.to_path_buf());
        if !path.starts_with(&root) {
            return Err(format!("Path escapes workspace: {rendered}"));
        }
        return Ok(Some(path));
    }
    Ok(None)
}

fn run_script_file(
    script_path: &Path,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    workspace_root: &Path,
) -> Result<(), String> {
    let body = std::fs::read_to_string(script_path).map_err(|e| {
        format!(
            "Failed to read postExecution script {}: {e}",
            script_path.display()
        )
    })?;
    let rendered_body = render_template(&body, args, maps, vars)?;
    let tmp = tempfile_script(workspace_root, &rendered_body)?;
    let result = run_shell_command(&format!("bash {}", shell_quote(&tmp)), workspace_root);
    let _ = std::fs::remove_file(&tmp);
    result.map_err(|e| format!("postExecution script {} failed: {e}", script_path.display()))
}

fn tempfile_script(workspace_root: &Path, body: &str) -> Result<PathBuf, String> {
    let dir = std::env::temp_dir().join(format!(
        "codemod_postexec_{}_{}",
        std::process::id(),
        workspace_root
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("ws")
    ));
    std::fs::create_dir_all(&dir).map_err(|e| format!("Failed to create temp dir: {e}"))?;
    let path = dir.join("script.sh");
    std::fs::write(&path, body).map_err(|e| format!("Failed to write temp script: {e}"))?;
    Ok(path)
}

fn shell_quote(path: &Path) -> String {
    let s = path.to_string_lossy();
    format!("'{}'", s.replace('\'', "'\\''"))
}

fn run_shell_command(command: &str, workspace_root: &Path) -> Result<(), String> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(workspace_root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map_err(|e| format!("Failed to run postExecution `{command}`: {e}"))?;
    if !status.success() {
        return Err(format!("postExecution failed (exit={status}): {command}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static N: AtomicUsize = AtomicUsize::new(0);

    fn temp_ws(name: &str) -> PathBuf {
        let n = N.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "codemod_postexec_test_{name}_{}_{n}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".codemod/scripts")).unwrap();
        dir
    }

    #[test]
    fn runs_shell_command_with_jinja() {
        let ws = temp_ws("shell");
        let marker = ws.join("out.txt");
        let mut args = BTreeMap::new();
        args.insert("name".into(), "world".into());
        run_post_execution(
            &[PostExecution::String(format!(
                "printf '{{{{ name }}}}' > '{}'",
                marker.display()
            ))],
            &args,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &ws,
            &ws.join(".codemod"),
            None,
        )
        .unwrap();
        assert_eq!(std::fs::read_to_string(marker).unwrap(), "world");
        let _ = std::fs::remove_dir_all(ws);
    }

    #[test]
    fn runs_script_with_jinja_body() {
        let ws = temp_ws("script");
        let script = ws.join(".codemod/scripts/hi.sh");
        std::fs::write(&script, "printf '{{ name }}' > out.txt\n").unwrap();
        let mut args = BTreeMap::new();
        args.insert("name".into(), "scripted".into());
        run_post_execution(
            &[PostExecution::String("scripts/hi.sh".into())],
            &args,
            &BTreeMap::new(),
            &BTreeMap::new(),
            &ws,
            &ws.join(".codemod"),
            None,
        )
        .unwrap();
        assert_eq!(
            std::fs::read_to_string(ws.join("out.txt")).unwrap(),
            "scripted"
        );
        let _ = std::fs::remove_dir_all(ws);
    }

    #[test]
    fn rejects_map_entries() {
        let ws = temp_ws("map");
        let err = run_post_execution(
            &[PostExecution::Map(serde_yaml::Value::Mapping(
                serde_yaml::Mapping::new(),
            ))],
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &ws,
            &ws.join(".codemod"),
            None,
        )
        .unwrap_err();
        assert!(err.contains("map/object"));
        let _ = std::fs::remove_dir_all(ws);
    }

    #[test]
    fn rejects_absolute_script_paths() {
        let ws = temp_ws("abs");
        let err = run_post_execution(
            &[PostExecution::String("/etc/passwd".into())],
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &ws,
            &ws.join(".codemod"),
            None,
        )
        .unwrap_err();
        assert!(err.contains("Absolute"));
        let _ = std::fs::remove_dir_all(ws);
    }

    #[test]
    fn prefers_recipe_local_script_over_codemod_fallback() {
        let ws = temp_ws("local_script");
        let recipe_dir = ws.join(".codemod/recipes");
        std::fs::create_dir_all(recipe_dir.join("scripts")).unwrap();
        std::fs::write(
            ws.join(".codemod/scripts/hi.sh"),
            "printf shared > out.txt\n",
        )
        .unwrap();
        std::fs::write(recipe_dir.join("scripts/hi.sh"), "printf local > out.txt\n").unwrap();
        let recipe_file = recipe_dir.join("demo.yaml");

        run_post_execution(
            &[PostExecution::String("scripts/hi.sh".into())],
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            &ws,
            &ws.join(".codemod"),
            Some(&recipe_file),
        )
        .unwrap();

        assert_eq!(
            std::fs::read_to_string(ws.join("out.txt")).unwrap(),
            "local"
        );
        let _ = std::fs::remove_dir_all(ws);
    }
}
