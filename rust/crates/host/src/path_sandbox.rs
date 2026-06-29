use crate::protocol::{DiagnosticSource, RecipeDiagnostic};
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("{message}")]
pub struct PathSandboxError {
    pub code: &'static str,
    pub message: String,
}

/// Validates that relative paths resolve inside the workspace root.
pub struct PathSandbox {
    workspace_root: PathBuf,
}

impl PathSandbox {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    pub fn resolve_workspace_relative(
        &self,
        relative_path: &str,
    ) -> Result<PathBuf, PathSandboxError> {
        let normalized = normalize(relative_path)?;
        if normalized.starts_with('/') {
            return Err(PathSandboxError {
                code: "E_PATH_TRAVERSAL",
                message: format!("Absolute paths are not allowed: {relative_path}"),
            });
        }

        let resolved = self.workspace_root.join(&normalized);
        let resolved = resolved.canonicalize().unwrap_or(resolved);
        let root = self
            .workspace_root
            .canonicalize()
            .unwrap_or_else(|_| self.workspace_root.clone());

        if !resolved.starts_with(&root) {
            return Err(PathSandboxError {
                code: "E_PATH_TRAVERSAL",
                message: format!("Path escapes workspace: {relative_path}"),
            });
        }

        Ok(resolved)
    }

    pub fn resolve_template_relative(
        &self,
        codemod_root: &str,
        relative_path: &str,
    ) -> Result<PathBuf, PathSandboxError> {
        let combined = format!("{codemod_root}/{relative_path}");
        self.resolve_workspace_relative(&combined)
    }
}

pub fn diagnostic_from_sandbox(error: PathSandboxError, file: &str) -> RecipeDiagnostic {
    RecipeDiagnostic {
        severity: "error",
        code: error.code,
        message: error.message,
        sources: vec![DiagnosticSource {
            file: file.to_string(),
            line: None,
            column: None,
        }],
    }
}

fn normalize(path: &str) -> Result<String, PathSandboxError> {
    let normalized = path.replace('\\', "/");
    let segments: Vec<&str> = normalized
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect();

    if segments.contains(&"..") {
        return Err(PathSandboxError {
            code: "E_PATH_TRAVERSAL",
            message: format!("Path must not contain \"..\": {path}"),
        });
    }

    Ok(segments.join("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_path_traversal_in_template_paths() {
        let sandbox = PathSandbox::new(std::env::temp_dir());
        let err = sandbox
            .resolve_template_relative(".codemod", "../outside.txt")
            .unwrap_err();
        assert_eq!(err.code, "E_PATH_TRAVERSAL");
    }

    #[test]
    fn resolves_paths_under_workspace() {
        let workspace =
            std::env::temp_dir().join(format!("codemod_sandbox_ok_{}", std::process::id()));
        std::fs::create_dir_all(&workspace).unwrap();
        let sandbox = PathSandbox::new(workspace.clone());
        let resolved = sandbox.resolve_workspace_relative("lib/main.dart").unwrap();
        assert!(resolved.starts_with(&workspace));
        let _ = std::fs::remove_dir_all(workspace);
    }
}
