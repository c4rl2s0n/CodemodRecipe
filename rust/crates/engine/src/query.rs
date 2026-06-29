use std::path::{Path, PathBuf};

use crate::engine::EngineError;

/// Resolve unified `query:` field — inline text or path to a `.scm` file.
pub fn resolve_query_source(
    query: &str,
    recipe_file: Option<&Path>,
    codemod_root: &Path,
) -> Result<String, EngineError> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Err(EngineError::Query("query must not be empty".to_string()));
    }

    if !looks_like_file_path(trimmed) {
        return Ok(trimmed.to_string());
    }

    for candidate in candidate_paths(trimmed, recipe_file, codemod_root) {
        if candidate.is_file() {
            return std::fs::read_to_string(&candidate).map_err(|e| {
                EngineError::Query(format!(
                    "failed to read query file {}: {e}",
                    candidate.display()
                ))
            });
        }
    }

    Err(EngineError::Query(format!(
        "query file not found: {trimmed}"
    )))
}

fn looks_like_file_path(query: &str) -> bool {
    query.ends_with(".scm")
        || query.contains('/')
        || query.contains('\\')
        || (query.ends_with(".yaml") && !query.contains('('))
}

fn candidate_paths(
    query: &str,
    recipe_file: Option<&Path>,
    codemod_root: &Path,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(recipe) = recipe_file.and_then(|p| p.parent()) {
        paths.push(recipe.join(query));
        paths.push(recipe.join("queries").join(query));
    }
    paths.push(codemod_root.join(query));
    if let Some(recipe) = recipe_file.and_then(|p| p.parent()) {
        paths.push(recipe.join("..").join(query));
    }
    paths.push(codemod_root.join("queries").join(query));
    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
    }

    #[test]
    fn reads_query_from_scm_file_relative_to_recipe() {
        let repo = repo_root();
        let recipe = repo.join("test/fixtures/rust_oracle/insert_log_line.recipe.yaml");
        let codemod = repo.join(".codemod");
        let text = resolve_query_source(
            "settings_update_body.scm",
            Some(&recipe),
            &codemod,
        )
        .unwrap();
        assert!(text.contains("class_declaration"));
        assert!(text.contains("@body"));
    }

    #[test]
    fn uses_inline_query_when_not_a_path() {
        let repo = repo_root();
        let inline = "(identifier) @x";
        let text = resolve_query_source(inline, None, &repo.join(".codemod")).unwrap();
        assert_eq!(text, inline);
    }

    #[test]
    fn errors_when_query_file_missing() {
        let repo = repo_root();
        let err = resolve_query_source("missing/file.scm", None, &repo.join(".codemod")).unwrap_err();
        assert!(err.to_string().contains("query file not found"));
    }
}
