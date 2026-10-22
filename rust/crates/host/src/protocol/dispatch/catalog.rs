//! Catalog / bootstrap / validate HostCommand handlers.

use crate::protocol::dispatch::common::{catalog_response, error_json, to_value};
use crate::protocol::DescribeResponse;
use crate::recipe::registry::RecipeRegistry;
use crate::recipe::validate::{validate_recipe, validate_workspace};

pub(super) fn reload(registry: &mut RecipeRegistry) -> serde_json::Value {
    registry.reload();
    catalog_response(registry)
}

pub(super) fn list(registry: &RecipeRegistry) -> serde_json::Value {
    catalog_response(registry)
}

pub(super) fn validate(registry: &mut RecipeRegistry, recipe: Option<String>) -> serde_json::Value {
    if let Some(recipe_id) = recipe {
        to_value(validate_recipe(registry, &recipe_id))
    } else {
        to_value(validate_workspace(registry))
    }
}

pub(super) fn describe(registry: &RecipeRegistry, recipe: String) -> serde_json::Value {
    match registry.get(&recipe) {
        Some(schema) => to_value(DescribeResponse {
            ok: true,
            error: None,
            recipe: Some(schema),
        }),
        None => to_value(DescribeResponse {
            ok: false,
            error: Some(format!("Recipe not found: {recipe}")),
            recipe: None,
        }),
    }
}

pub(super) fn bootstrap(
    registry: &RecipeRegistry,
    force: bool,
    edit_policy: Option<String>,
    companions: Vec<String>,
) -> serde_json::Value {
    let policy = match crate::config::bootstrap::EditPolicy::parse(
        edit_policy.as_deref().unwrap_or("recommend"),
    ) {
        Ok(p) => p,
        Err(e) => return error_json(e),
    };
    let companions = match crate::config::bootstrap::parse_companions(&companions) {
        Ok(c) => c,
        Err(e) => return error_json(e),
    };
    crate::config::bootstrap::bootstrap_project(
        &registry.workspace_root,
        force,
        policy,
        &companions,
    )
}

pub(super) fn run_tests(registry: &mut RecipeRegistry, filter: Option<String>) -> serde_json::Value {
    to_value(crate::testing::run_tests(registry, filter.as_deref()))
}
