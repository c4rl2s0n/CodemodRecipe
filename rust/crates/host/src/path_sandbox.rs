use crate::diag_source::source_file_only;
use crate::protocol::RecipeDiagnostic;
use codemod_recipe_core::resource_path::{resolve_under_root, ResourcePathError};
use std::path::PathBuf;

pub type PathSandboxError = ResourcePathError;

/// Validates that relative paths resolve inside the workspace root.
pub struct PathSandbox {
    workspace_root: PathBuf,
}

impl PathSandbox {
    pub fn new(workspace_root: PathBuf) -> Self {
        Self { workspace_root }
    }

    pub fn workspace_root(&self) -> &std::path::Path {
        &self.workspace_root
    }

    pub fn resolve_workspace_relative(
        &self,
        relative_path: &str,
    ) -> Result<PathBuf, PathSandboxError> {
        resolve_under_root(&self.workspace_root, relative_path)
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
    RecipeDiagnostic::simple(
        "error",
        error.code,
        error.message,
        vec![source_file_only(file)],
    )
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
