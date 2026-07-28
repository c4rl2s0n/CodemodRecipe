use std::path::{Path, PathBuf};

pub mod recipe_keys {
    pub const ID: &str = "id";
    pub const WITH: &str = "with";
    pub const STEPS: &str = "steps";
    pub const QUERIES: &str = "queries";
    pub const MAP: &str = "map";
    pub const VALUES: &str = "values";
    pub const POST_EXECUTION: &str = "postExecution";
    pub const INPUT_KIND: &str = "inputKind";
    pub const DEFAULTS_TO: &str = "defaultsTo";
    pub const ALLOW_CUSTOM_VALUE: &str = "allowCustomValue";
    pub const CONTEXT_KEY: &str = "contextKey";
    pub const TEMPLATE_FILE: &str = "templateFile";
    pub const IF_EXISTS: &str = "ifExists";
    pub const IF_MISSING: &str = "ifMissing";
    pub const WHEN_NOT: &str = "whenNot";
    pub const INCLUDE_LEADING_TRIVIA: &str = "includeLeadingTrivia";
    pub const LET: &str = "let";

    pub const RECIPE: &str = "recipe";
    pub const EDIT: &str = "edit";
    pub const CREATE: &str = "create";
    pub const DELETE: &str = "delete";
    pub const INSERT: &str = "insert";
    pub const REPLACE: &str = "replace";
    pub const REMOVE: &str = "remove";

    pub const PATH: &str = "path";
    pub const LANGUAGE: &str = "language";
    pub const QUERY: &str = "query";
    pub const CAPTURE: &str = "capture";
    pub const TEXT: &str = "text";
    pub const NAME: &str = "name";
}

pub mod query_conventions {
    use super::*;

    pub const QUERIES_DIR: &str = "queries";
    pub const QUERY_FILE_EXT: &str = ".scm";
    pub const YAML_FILE_EXT: &str = ".yaml";

    pub fn looks_like_query_path(query: &str) -> bool {
        let trimmed = query.trim();
        if trimmed.contains('(') {
            return false;
        }
        trimmed.ends_with(QUERY_FILE_EXT)
            || trimmed.contains('/')
            || trimmed.contains('\\')
            || (trimmed.ends_with(YAML_FILE_EXT) && !trimmed.contains('('))
    }

    pub fn candidate_query_paths(
        query: &str,
        recipe_file: Option<&Path>,
        codemod_root: &Path,
    ) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        if let Some(recipe_dir) = recipe_file.and_then(|p| p.parent()) {
            paths.push(recipe_dir.join(query));
            paths.push(recipe_dir.join(QUERIES_DIR).join(query));
        }
        paths.push(codemod_root.join(query));
        if let Some(recipe_dir) = recipe_file.and_then(|p| p.parent()) {
            paths.push(recipe_dir.join("..").join(query));
        }
        paths.push(codemod_root.join(QUERIES_DIR).join(query));
        paths
    }
}

pub mod preview_kinds {
    pub const EDIT: &str = "edit";
    pub const CREATE: &str = "create";
    pub const DELETE: &str = "delete";
}
