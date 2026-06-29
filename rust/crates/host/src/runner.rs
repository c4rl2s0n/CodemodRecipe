use codemod_recipe_core::patch::{apply_patches, SourcePatch};
use codemod_recipe_engine::engine::{Engine, EngineError, QueryContext};
use codemod_recipe_yaml::model::Recipe;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::registry::{render_recipe_templates, RecipeRegistry};

pub struct RunResult {
    pub modified: String,
    pub patches: Vec<SourcePatch>,
}

pub fn validate_required_args(
    recipe_id: &str,
    args: &BTreeMap<String, String>,
    registry: &RecipeRegistry,
) -> Result<(), String> {
    let schema = registry
        .get(recipe_id)
        .ok_or_else(|| format!("Recipe not found: {recipe_id}"))?;
    let missing: Vec<String> = schema
        .args
        .iter()
        .filter(|arg| arg.required && !args.contains_key(&arg.name))
        .map(|arg| arg.name.clone())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "Missing required arguments: {}",
            missing.join(", ")
        ));
    }
    Ok(())
}

pub fn run_recipe_on_source(
    recipe: &Recipe,
    file_arg: &str,
    source: &str,
    ctx: &QueryContext<'_>,
) -> Result<RunResult, EngineError> {
    let mut engine = Engine::new_dart()?;
    let rendered = render_recipe_templates(recipe, &BTreeMap::new(), &BTreeMap::new());
    let result = engine.apply_recipe_to_source(ctx, &rendered, file_arg, source)?;
    Ok(RunResult {
        modified: result.modified,
        patches: result.patches,
    })
}

pub fn run_recipe_on_file(
    registry: &RecipeRegistry,
    recipe_id: &str,
    args: &BTreeMap<String, String>,
) -> Result<(String, String, RunResult), String> {
    validate_required_args(recipe_id, args, registry)?;

    let file = args
        .get("file")
        .cloned()
        .ok_or_else(|| "Missing required arg: file".to_string())?;

    let (recipe_ast, recipe_path) = registry.load_recipe_ast(recipe_id)?;
    let merged_maps = registry.merged_maps_for(&recipe_ast);
    let file_path = registry.resolve_file_path(&file);
    let before =
        std::fs::read_to_string(&file_path).map_err(|e| format!("Failed to read {file}: {e}"))?;

    let rendered = render_recipe_templates(&recipe_ast, args, &merged_maps);
    let ctx = QueryContext {
        recipe_file: Some(recipe_path.as_path()),
        codemod_root: registry.codemod_root(),
    };
    let mut engine = Engine::new_dart().map_err(|e| e.to_string())?;
    let result = engine
        .apply_recipe_to_source(&ctx, &rendered, &file, &before)
        .map_err(|e| e.to_string())?;

    Ok((
        file,
        before,
        RunResult {
            modified: result.modified,
            patches: result.patches,
        },
    ))
}

pub fn apply_patches_to_source(source: &str, patches: &[SourcePatch]) -> Result<String, String> {
    apply_patches(source, patches).map_err(|e| e.to_string())
}

pub fn write_file(path: &Path, contents: &str) -> Result<(), String> {
    std::fs::write(path, contents).map_err(|e| format!("Failed to write {}: {e}", path.display()))
}

pub fn snapshot_paths_for_args(
    registry: &RecipeRegistry,
    args: &BTreeMap<String, String>,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(file) = args.get("file") {
        paths.push(registry.resolve_file_path(file));
    }
    paths
}
