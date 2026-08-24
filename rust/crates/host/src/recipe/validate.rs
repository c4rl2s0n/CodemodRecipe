use crate::protocol::ValidateResponse;
use crate::recipe::registry::RecipeRegistry;

pub fn validate_workspace(registry: &mut RecipeRegistry) -> ValidateResponse {
    registry.reload();
    let (_recipes, diagnostics) = registry.list();
    ValidateResponse {
        ok: diagnostics.iter().all(|d| d.severity != "error"),
        error: None,
        diagnostics: if diagnostics.is_empty() {
            None
        } else {
            Some(diagnostics)
        },
    }
}

pub fn validate_recipe(registry: &RecipeRegistry, recipe_id: &str) -> ValidateResponse {
    match registry.get(recipe_id) {
        Some(_) => ValidateResponse {
            ok: true,
            error: None,
            diagnostics: None,
        },
        None => ValidateResponse {
            ok: false,
            error: Some(format!("Recipe not found: {recipe_id}")),
            diagnostics: None,
        },
    }
}
