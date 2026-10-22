//! Shared query specifier (engine + host conversion). No tree-sitter / Luau deps.

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
