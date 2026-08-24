use tree_sitter::Language;

/// Native grammar overrides not available in tree-sitter-language-pack.
pub fn native_language(id: &str) -> Option<Language> {
    match id {
        #[cfg(feature = "lang-sqlite")]
        "sqlite" => Some(tree_sitter_sqlite3::LANGUAGE.into()),
        #[cfg(feature = "lang-postgres")]
        "postgres" => Some(tree_sitter_postgres::LANGUAGE.into()),
        _ => None,
    }
}

pub fn native_language_ids() -> &'static [&'static str] {
    &[
        #[cfg(feature = "lang-sqlite")]
        "sqlite",
        #[cfg(feature = "lang-postgres")]
        "postgres",
    ]
}
