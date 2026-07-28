pub mod query_conventions {
    pub const QUERY_FILE_EXT: &str = ".scm";
    pub const YAML_FILE_EXT: &str = ".yaml";
    pub const YML_FILE_EXT: &str = ".yml";
    pub const QUERIES_DIR: &str = "queries";

    pub fn looks_like_query_path(query: &str) -> bool {
        let trimmed = query.trim();
        if trimmed.contains('(') {
            return false;
        }
        if trimmed.ends_with(YAML_FILE_EXT) || trimmed.ends_with(YML_FILE_EXT) {
            return false;
        }
        trimmed.ends_with(QUERY_FILE_EXT) || trimmed.contains('/') || trimmed.contains('\\')
    }
}

#[cfg(test)]
mod query_conventions_tests {
    use super::query_conventions::looks_like_query_path;

    #[test]
    fn yaml_paths_are_not_scm_file_paths() {
        assert!(!looks_like_query_path("queries/shared.yaml"));
        assert!(!looks_like_query_path("lib.yml"));
    }

    #[test]
    fn scm_and_slash_paths_are_file_backed() {
        assert!(looks_like_query_path("foo.scm"));
        assert!(looks_like_query_path("queries/foo.scm"));
    }
}

pub mod preview_kinds {
    pub use crate::dsl::recipe::steps::create::WIRE as CREATE;
    pub use crate::dsl::recipe::steps::delete::WIRE as DELETE;
    pub use crate::dsl::recipe::steps::edit::WIRE as EDIT;
}

pub use crate::dsl;
