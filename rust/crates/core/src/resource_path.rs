use std::path::{Path, PathBuf};

use thiserror::Error;

pub const PATH_TRAVERSAL_CODE: &str = "E_PATH_TRAVERSAL";

#[derive(Debug, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct ResourcePathError {
    pub code: &'static str,
    pub message: String,
}

pub fn normalize_relative_path(path: &str) -> Result<String, ResourcePathError> {
    let normalized = path.replace('\\', "/");
    if normalized.starts_with('/') {
        return Err(ResourcePathError {
            code: PATH_TRAVERSAL_CODE,
            message: format!("Absolute paths are not allowed: {path}"),
        });
    }
    let segments: Vec<&str> = normalized
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect();

    if segments.contains(&"..") {
        return Err(ResourcePathError {
            code: PATH_TRAVERSAL_CODE,
            message: format!("Path must not contain \"..\": {path}"),
        });
    }

    Ok(segments.join("/"))
}

pub fn resolve_under_root(root: &Path, relative_path: &str) -> Result<PathBuf, ResourcePathError> {
    let normalized = normalize_relative_path(relative_path)?;
    let canonical_root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let resolved = canonical_root.join(&normalized);
    let canonical_resolved = resolved.canonicalize().unwrap_or_else(|_| resolved.clone());
    if !canonical_resolved.starts_with(&canonical_root) {
        return Err(ResourcePathError {
            code: PATH_TRAVERSAL_CODE,
            message: format!("Path escapes root: {relative_path}"),
        });
    }

    Ok(canonical_resolved)
}

pub fn resource_candidate_paths(
    relative_path: &str,
    referrer_file: Option<&Path>,
    fallback_root: &Path,
    conventional_subdir: Option<&str>,
) -> Result<Vec<PathBuf>, ResourcePathError> {
    let normalized = normalize_relative_path(relative_path)?;
    let include_subdir = conventional_subdir.is_some() && !normalized.contains('/');
    let mut roots = Vec::new();
    if let Some(referrer_dir) = referrer_file.and_then(|path| path.parent()) {
        roots.push(referrer_dir.to_path_buf());
        if include_subdir {
            roots.push(referrer_dir.join(conventional_subdir.unwrap()));
        }
    }
    roots.push(fallback_root.to_path_buf());
    if include_subdir {
        roots.push(fallback_root.join(conventional_subdir.unwrap()));
    }

    let mut paths = Vec::new();
    for root in roots {
        let resolved = resolve_under_root(&root, &normalized)?;
        if !paths.contains(&resolved) {
            paths.push(resolved);
        }
    }
    Ok(paths)
}

pub fn resolve_existing_resource(
    relative_path: &str,
    referrer_file: Option<&Path>,
    fallback_root: &Path,
    conventional_subdir: Option<&str>,
) -> Result<Option<PathBuf>, ResourcePathError> {
    for candidate in resource_candidate_paths(
        relative_path,
        referrer_file,
        fallback_root,
        conventional_subdir,
    )? {
        if candidate.is_file() {
            return Ok(Some(candidate));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "codemod_resource_path_{name}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn resolves_under_root() {
        let root = temp_dir("root");
        let path = resolve_under_root(&root, "lib/main.dart").unwrap();
        assert!(path.starts_with(&root));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_traversal() {
        let root = temp_dir("traversal");
        let err = resolve_under_root(&root, "../outside.txt").unwrap_err();
        assert_eq!(err.code, PATH_TRAVERSAL_CODE);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn rejects_absolute_paths() {
        let root = temp_dir("absolute");
        let err = resolve_under_root(&root, "/etc/passwd").unwrap_err();
        assert_eq!(err.code, PATH_TRAVERSAL_CODE);
        assert!(err.message.contains("Absolute"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn prefers_referrer_relative_file() {
        let workspace = temp_dir("prefer_local");
        let codemod_root = workspace.join(".codemod");
        let recipe_dir = codemod_root.join("recipes");
        std::fs::create_dir_all(recipe_dir.join("templates")).unwrap();
        std::fs::create_dir_all(codemod_root.join("templates")).unwrap();
        let recipe_file = recipe_dir.join("feature.yaml");
        std::fs::write(recipe_dir.join("templates/widget.template"), "local").unwrap();
        std::fs::write(codemod_root.join("templates/widget.template"), "shared").unwrap();

        let resolved = resolve_existing_resource(
            "templates/widget.template",
            Some(&recipe_file),
            &codemod_root,
            None,
        )
        .unwrap()
        .unwrap();

        assert_eq!(std::fs::read_to_string(resolved).unwrap(), "local");
        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn uses_conventional_subdir_for_bare_names() {
        let workspace = temp_dir("queries");
        let codemod_root = workspace.join(".codemod");
        let recipe_dir = workspace.join("recipes");
        std::fs::create_dir_all(recipe_dir.join("queries")).unwrap();
        let recipe_file = recipe_dir.join("feature.yaml");
        std::fs::write(recipe_dir.join("queries/body.scm"), "(identifier) @x").unwrap();

        let resolved = resolve_existing_resource(
            "body.scm",
            Some(&recipe_file),
            &codemod_root,
            Some("queries"),
        )
        .unwrap()
        .unwrap();

        assert_eq!(resolved, recipe_dir.join("queries/body.scm"));
        let _ = std::fs::remove_dir_all(workspace);
    }
}
