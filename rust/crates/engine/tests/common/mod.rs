use codemod_recipe_engine::engine::Engine;
use codemod_recipe_engine::registry::{ensure_language_downloaded, LanguageRegistry};

pub fn engine_for(language_id: &str) -> (LanguageRegistry, String) {
    ensure_language_downloaded(language_id);
    let mut registry = LanguageRegistry::new();
    registry
        .get(language_id)
        .expect("language should load in tests");
    (registry, language_id.to_string())
}

pub fn with_engine<F, R>(language_id: &str, f: F) -> R
where
    F: FnOnce(&mut Engine) -> R,
{
    let (mut registry, id) = engine_for(language_id);
    let engine = registry.get(&id).expect("engine");
    f(engine)
}
