pub mod engine;
pub mod language;
pub mod ops;
pub mod query;
pub mod span;

pub use language::registry::{
    ensure_language_downloaded, is_known_language, language_from_extension, LanguageRegistry,
    RegistryConfig,
};
pub use ops::{
    looks_like_query_path, EditOp, InsertAnchor, InsertOp, QuerySpec, RemoveOp, ReplaceOp,
    QUERIES_DIR,
};
