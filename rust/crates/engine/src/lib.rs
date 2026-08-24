pub mod engine;
pub mod language;
pub mod query;
pub mod span;

pub use language::registry::{
    ensure_language_downloaded, is_known_language, language_from_extension, LanguageRegistry,
    RegistryConfig,
};
