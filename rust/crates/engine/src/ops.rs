//! Engine-native query specs and edit operations (no recipe DSL).

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QuerySpec {
    Single(String),
    Chain(Vec<String>),
}

impl QuerySpec {
    pub fn single(text: impl Into<String>) -> Self {
        QuerySpec::Single(text.into())
    }

    pub fn steps(&self) -> Vec<&str> {
        match self {
            QuerySpec::Single(s) => vec![s.as_str()],
            QuerySpec::Chain(v) => v.iter().map(String::as_str).collect(),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            QuerySpec::Single(s) => s.trim().is_empty(),
            QuerySpec::Chain(v) => v.is_empty() || v.iter().all(|s| s.trim().is_empty()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InsertAnchor {
    Start,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertOp {
    pub query: QuerySpec,
    pub capture: String,
    pub anchor: InsertAnchor,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceOp {
    pub query: QuerySpec,
    pub capture: String,
    pub text: String,
    pub include_leading_trivia: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveOp {
    pub query: QuerySpec,
    pub capture: String,
    pub include_leading_trivia: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditOp {
    Insert(InsertOp),
    Replace(ReplaceOp),
    Remove(RemoveOp),
}

pub const QUERY_FILE_EXT: &str = ".scm";
pub const QUERIES_DIR: &str = "queries";

/// True when `query` looks like a `.scm` path rather than an inline S-expression.
pub fn looks_like_query_path(query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.contains('(') {
        return false;
    }
    if trimmed.ends_with(".yaml") || trimmed.ends_with(".yml") {
        return false;
    }
    trimmed.ends_with(QUERY_FILE_EXT) || trimmed.contains('/') || trimmed.contains('\\')
}

#[cfg(test)]
mod tests {
    use super::looks_like_query_path;

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
