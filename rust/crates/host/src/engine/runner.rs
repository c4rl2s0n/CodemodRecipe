use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use codemod_recipe_core::apply::file_change::FileChange;
use codemod_recipe_lua::load::load_recipe;
use codemod_recipe_lua::model::Recipe;

use crate::lua_runtime::{run_recipe, RecipeRunResult, RunKind};
use crate::recipe::args;
use crate::recipe::registry::RecipeRegistry;
use crate::sandbox::path_sandbox::PathSandbox;

pub struct CollectedChanges {
    pub recipe: Recipe,
    pub recipe_path: Option<PathBuf>,
    pub changes: Vec<FileChange>,
    pub transcript: Vec<codemod_recipe_lua::TranscriptEvent>,
    pub saw_post_exec: bool,
}

pub fn parse_inline_recipe(value: &serde_json::Value) -> Result<Recipe, String> {
    let source = if let Some(s) = value.as_str() {
        s.to_string()
    } else if let Some(s) = value.get("source").and_then(|v| v.as_str()) {
        s.to_string()
    } else {
        return Err("inlineRecipe must be a Luau source string".to_string());
    };
    load_recipe(&source, "<inline>").map_err(|e| e.to_string())
}

pub fn resolve_recipe(
    registry: &RecipeRegistry,
    recipe_id: Option<&str>,
    inline_recipe: Option<&serde_json::Value>,
) -> Result<(Recipe, Option<PathBuf>), String> {
    match (recipe_id, inline_recipe) {
        (Some(id), None) => registry.load_recipe_ast(id).map(|(r, p)| (r, Some(p))),
        (None, Some(value)) => Ok((parse_inline_recipe(value)?, None)),
        (Some(_), Some(_)) => Err("Provide either recipe or inlineRecipe, not both".to_string()),
        (None, None) => Err("Missing recipe or inlineRecipe".to_string()),
    }
}

pub fn collect_recipe_changes(
    registry: &RecipeRegistry,
    recipe: &Recipe,
    _recipe_path: Option<&Path>,
    args: &BTreeMap<String, serde_json::Value>,
    kind: RunKind,
) -> Result<CollectedChanges, String> {
    let effective = args::validate_typed_args(
        recipe,
        registry.types_by_id(),
        registry.maps_by_id(),
        args,
    )?;
    let mut maps = registry.maps_by_id().clone();
    for (k, v) in &recipe.maps {
        maps.entry(k.clone()).or_insert_with(|| v.clone());
    }
    let mut queries = registry.queries_by_id().clone();
    queries.extend(recipe.queries.clone());
    let result: RecipeRunResult = run_recipe(
        kind,
        recipe,
        &effective,
        maps,
        registry.vars_by_id().clone(),
        queries,
        registry.recipes_map().clone(),
        registry.workspace_root.clone(),
        registry.language_registry.config().clone(),
        registry.codemod_root().to_path_buf(),
    )?;
    Ok(CollectedChanges {
        recipe: recipe.clone(),
        recipe_path: recipe.path.clone(),
        changes: result.changes,
        transcript: result.transcript,
        saw_post_exec: result.saw_post_exec,
    })
}

pub fn planned_snapshot_paths(workspace: &Path, changes: &[FileChange]) -> Vec<PathBuf> {
    let sandbox = PathSandbox::new(workspace.to_path_buf());
    changes
        .iter()
        .filter_map(|c| sandbox.resolve_workspace_relative(c.path()).ok())
        .collect()
}
