//! Shared dispatch helpers.

use std::collections::BTreeMap;

use crate::protocol::keys as protocol_keys;
use crate::protocol::RecipeCatalogResponse;
use crate::recipe::registry::RecipeRegistry;

pub(super) struct RecipeRequest<'a> {
    pub recipe_id: Option<&'a str>,
    pub inline_recipe: Option<&'a serde_json::Value>,
    pub args: &'a BTreeMap<String, serde_json::Value>,
}

impl<'a> RecipeRequest<'a> {
    pub fn recipe_key(&self) -> String {
        if let Some(id) = self.recipe_id {
            id.to_string()
        } else if let Some(inline) = self.inline_recipe {
            inline
                .get(protocol_keys::ID)
                .and_then(|v| v.as_str())
                .unwrap_or(protocol_keys::INLINE_RECIPE_ID)
                .to_string()
        } else {
            protocol_keys::UNKNOWN_RECIPE_ID.to_string()
        }
    }
}

pub(super) fn error_json(message: impl Into<String>) -> serde_json::Value {
    let mut value = serde_json::Map::new();
    value.insert(
        protocol_keys::OK.to_string(),
        serde_json::Value::Bool(false),
    );
    value.insert(
        protocol_keys::ERROR.to_string(),
        serde_json::Value::String(message.into()),
    );
    serde_json::Value::Object(value)
}

pub(super) fn catalog_response(registry: &RecipeRegistry) -> serde_json::Value {
    let (recipes, diagnostics) = registry.list();
    to_value(RecipeCatalogResponse {
        ok: true,
        error: None,
        recipes: Some(recipes),
        diagnostics: if diagnostics.is_empty() {
            None
        } else {
            Some(diagnostics)
        },
        maps_loaded: Some(registry.maps_count()),
        map_ids: Some(registry.map_ids()),
        var_ids: Some(registry.var_ids()),
        type_ids: Some(registry.type_ids()),
        types: Some(registry.type_schemas()),
        language_ids: Some(
            codemod_recipe_engine::language::native::native_language_ids()
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        ),
        query_ids: Some(registry.queries_by_id().keys().cloned().collect()),
        queries: Some(registry.queries_by_id().clone()),
        components: Some(registry.component_schemas()),
        stack: registry.stack_schema(),
        test_suites: Some(registry.test_suite_schemas()),
    })
}
pub(super) fn write_workspace_relative(
    registry: &RecipeRegistry,
    relative: Option<&str>,
    bytes: &[u8],
) -> Result<Option<String>, String> {
    let Some(relative) = relative.filter(|s| !s.is_empty()) else {
        return Ok(None);
    };
    let abs =
        crate::sandbox::path_sandbox::PathSandbox::new(registry.workspace_root().to_path_buf())
            .resolve_workspace_relative(relative)
            .map_err(|e| e.message)?;
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&abs, bytes).map_err(|e| e.to_string())?;
    Ok(Some(relative.to_string()))
}
pub(super) fn to_value<T: serde::Serialize>(value: T) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or_else(|e| error_json(format!("serialization failed: {e}")))
}

