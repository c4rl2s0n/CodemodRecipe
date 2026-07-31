use std::collections::HashMap;
use std::path::Path;

use crate::adapter::adapter_for_language;
use crate::engine::{Engine, EngineError};
use crate::native;

#[derive(Debug, Clone)]
pub struct RegistryConfig {
    /// Default language when extension is `.sql` and `language:` is omitted.
    pub sql_default: String,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            sql_default: "sqlite".to_string(),
        }
    }
}

pub struct LanguageRegistry {
    engines: HashMap<String, Engine>,
    config: RegistryConfig,
}

impl LanguageRegistry {
    pub fn new() -> Self {
        Self::with_config(RegistryConfig::default())
    }

    pub fn with_config(config: RegistryConfig) -> Self {
        Self {
            engines: HashMap::new(),
            config,
        }
    }

    pub fn config(&self) -> &RegistryConfig {
        &self.config
    }

    pub fn get(&mut self, language_id: &str) -> Result<&mut Engine, EngineError> {
        let id = language_id.to_string();
        if !self.engines.contains_key(&id) {
            let language = load_language(&id)?;
            let adapter = adapter_for_language(&id, language);
            let engine = Engine::new(adapter)?;
            self.engines.insert(id.clone(), engine);
        }
        self.engines
            .get_mut(&id)
            .ok_or_else(|| EngineError::LanguageLoad(format!("failed to cache engine for {id}")))
    }

    pub fn resolve_language_id(
        &self,
        explicit: Option<&str>,
        file_path: &str,
    ) -> Result<String, EngineError> {
        if let Some(lang) = explicit {
            let lang = lang.trim();
            if lang.is_empty() {
                return Err(EngineError::LanguageLoad(
                    "edit.language must not be empty".to_string(),
                ));
            }
            if !is_known_language(lang) {
                return Err(EngineError::LanguageNotSupported(lang.to_string()));
            }
            return Ok(lang.to_string());
        }

        if let Some(lang) = language_from_extension(file_path, &self.config) {
            if !is_known_language(&lang) {
                return Err(EngineError::LanguageNotSupported(lang));
            }
            return Ok(lang);
        }

        Err(EngineError::FileTypeNotSupported(file_path.to_string()))
    }

    pub fn resolve_for_edit(
        &mut self,
        explicit: Option<&str>,
        file_path: &str,
    ) -> Result<&mut Engine, EngineError> {
        let id = self.resolve_language_id(explicit, file_path)?;
        self.get(&id)
    }
}

impl Default for LanguageRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn load_language(id: &str) -> Result<tree_sitter::Language, EngineError> {
    if let Some(language) = native::native_language(id) {
        return Ok(language);
    }

    #[cfg(feature = "lang-pack")]
    {
        tree_sitter_language_pack::get_language(id)
            .map_err(|e| EngineError::LanguageLoad(e.to_string()))
    }

    #[cfg(not(feature = "lang-pack"))]
    {
        let _ = id;
        Err(EngineError::LanguageLoad(format!(
            "language {id} requires lang-pack feature"
        )))
    }
}

pub fn language_from_extension(file_path: &str, config: &RegistryConfig) -> Option<String> {
    let path = Path::new(file_path);
    let ext = path.extension()?.to_str()?;

    #[cfg(feature = "lang-pack")]
    {
        if ext == "sql" {
            return Some(config.sql_default.clone());
        }
        if let Some(lang) = tree_sitter_language_pack::detect_language_from_extension(ext) {
            return Some(lang.to_string());
        }
    }

    #[cfg(not(feature = "lang-pack"))]
    {
        let _ = config;
        match ext {
            "dart" => return Some("dart".to_string()),
            "rs" => return Some("rust".to_string()),
            "java" => return Some("java".to_string()),
            "kt" | "kts" => return Some("kotlin".to_string()),
            "sql" => return Some("sqlite".to_string()),
            "bq" => return Some("sql_bigquery".to_string()),
            _ => {}
        }
    }

    None
}

pub fn is_known_language(id: &str) -> bool {
    if native::native_language_ids().contains(&id) {
        return true;
    }

    #[cfg(feature = "lang-pack")]
    {
        // has_language includes the downloadable catalog (KNOWN_LANGUAGES), not
        // only grammars already present on disk (unlike available_languages).
        tree_sitter_language_pack::has_language(id)
    }

    #[cfg(not(feature = "lang-pack"))]
    {
        let _ = id;
        false
    }
}

pub fn ensure_language_downloaded(id: &str) {
    #[cfg(feature = "lang-pack")]
    {
        // prefetch downloads when the grammar is not loadable from disk;
        // download() skips when has_language is true even if the .so is missing.
        let _ = tree_sitter_language_pack::prefetch(&[id]);
    }
    let _ = id;
}

#[cfg(test)]
pub fn test_engine_for(language_id: &str) -> (LanguageRegistry, String) {
    ensure_language_downloaded(language_id);
    let mut registry = LanguageRegistry::new();
    registry
        .get(language_id)
        .expect("language should load in tests");
    (registry, language_id.to_string())
}
