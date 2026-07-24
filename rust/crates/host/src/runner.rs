use codemod_recipe_core::file_change::{merge_file_changes, FileChange, IfExists, IfMissing};
use codemod_recipe_core::patch::apply_patches;
use codemod_recipe_engine::engine::{EngineError, QueryContext};
use codemod_recipe_engine::LanguageRegistry;
use codemod_recipe_yaml::model::{
    CreateStep, DeleteStep, IfExistsStrategy, IfMissingStrategy, Recipe, Step,
};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::args;
use crate::path_sandbox::PathSandbox;
use crate::registry::{render_recipe_templates_with_root, RecipeRegistry};
use crate::template::render_template_file;

pub struct CollectedChanges {
    pub recipe: Recipe,
    pub recipe_path: Option<PathBuf>,
    pub changes: Vec<FileChange>,
}

pub fn parse_inline_recipe(value: &serde_json::Value) -> Result<Recipe, String> {
    serde_json::from_value(value.clone()).map_err(|e| format!("Invalid inlineRecipe: {e}"))
}

pub fn resolve_recipe(
    registry: &RecipeRegistry,
    recipe_id: Option<&str>,
    inline_recipe: Option<&serde_json::Value>,
) -> Result<(Recipe, Option<PathBuf>), String> {
    match (recipe_id, inline_recipe) {
        (Some(id), None) => registry.load_recipe_ast(id).map(|(r, p)| (r, Some(p))),
        (None, Some(value)) => {
            let recipe = parse_inline_recipe(value)?;
            Ok((recipe, None))
        }
        (Some(_), Some(_)) => Err("Provide either recipe or inlineRecipe, not both".to_string()),
        (None, None) => Err("Missing recipe or inlineRecipe".to_string()),
    }
}

pub fn validate_required_args(
    recipe: &Recipe,
    caller: &BTreeMap<String, String>,
) -> Result<(), String> {
    args::validate_required_args(recipe, caller).map(|_| ())
}

pub fn collect_recipe_changes(
    registry: &RecipeRegistry,
    recipe: &Recipe,
    recipe_path: Option<&Path>,
    args: &BTreeMap<String, String>,
) -> Result<CollectedChanges, String> {
    let effective = args::validate_required_args(recipe, args)?;
    let merged_maps = registry.merged_maps_for(recipe);
    let vars = registry.vars_by_id();
    let rendered = render_recipe_templates_with_root(
        recipe,
        &effective,
        &merged_maps,
        vars,
        Some(registry.codemod_root()),
    )?;

    let sandbox = PathSandbox::new(registry.workspace_root.clone());
    let codemod_rel = relative_codemod_path(registry);
    let mut raw_changes: Vec<FileChange> = Vec::new();
    let mut language_registry =
        LanguageRegistry::with_config(registry.language_config.clone());
    let ctx = QueryContext {
        recipe_file: recipe_path,
        codemod_root: registry.codemod_root(),
    };

    for step in &rendered.steps {
        match step {
            Step::Edit(edit) => {
                let relative = edit.path.clone();
                let absolute = sandbox
                    .resolve_workspace_relative(&relative)
                    .map_err(|e| e.message)?;
                let source = std::fs::read_to_string(&absolute)
                    .map_err(|e| format!("Failed to read {relative}: {e}"))?;
                let engine = language_registry
                    .resolve_for_edit(edit.language.as_deref(), &relative)
                    .map_err(engine_error_to_string)?;
                let patches = engine
                    .collect_patches_for_edit(&ctx, edit, &source)
                    .map_err(engine_error_to_string)?;
                if !patches.is_empty() {
                    raw_changes.push(FileChange::Patch {
                        path: relative,
                        source,
                        patches,
                    });
                }
            }
            Step::Create(create) => {
                raw_changes.push(collect_create_change(
                    registry,
                    &sandbox,
                    &codemod_rel,
                    create,
                    &effective,
                    &merged_maps,
                    vars,
                )?);
            }
            Step::Delete(delete) => {
                raw_changes.push(collect_delete_change(&sandbox, delete)?);
            }
            Step::RecipeRef(_) | Step::Scoped(_) => {}
            Step::Unknown(kind, _) => {
                return Err(format!("Unsupported step kind: {kind}"));
            }
        }
    }

    let changes = merge_file_changes(raw_changes)?;
    Ok(CollectedChanges {
        recipe: rendered,
        recipe_path: recipe_path.map(Path::to_path_buf),
        changes,
    })
}

fn collect_create_change(
    registry: &RecipeRegistry,
    sandbox: &PathSandbox,
    _codemod_rel: &str,
    create: &CreateStep,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<FileChange, String> {
    let relative = create.path.clone();
    let absolute = sandbox
        .resolve_workspace_relative(&relative)
        .map_err(|e| e.message)?;
    let exists = absolute.exists();

    let if_exists = match create.if_exists {
        IfExistsStrategy::Fail => IfExists::Fail,
        IfExistsStrategy::Skip => IfExists::Skip,
    };

    if exists && if_exists == IfExists::Fail {
        return Err(format!("File already exists: {relative}"));
    }
    if exists && if_exists == IfExists::Skip {
        return Ok(FileChange::Create {
            path: relative,
            content: String::new(),
            if_exists,
            format: create.format,
            skipped: true,
        });
    }

    let content = if let Some(inline) = &create.template {
        // Already rendered under the correct (possibly scoped) arg overlay.
        inline.clone()
    } else if let Some(file) = &create.template_file {
        render_template_file(file, args, maps, vars, registry.codemod_root())?
    } else {
        return Err("create step missing template".to_string());
    };
    Ok(FileChange::Create {
        path: relative,
        content,
        if_exists,
        format: create.format,
        skipped: false,
    })
}

fn collect_delete_change(
    sandbox: &PathSandbox,
    delete: &DeleteStep,
) -> Result<FileChange, String> {
    let relative = delete.path.clone();
    let absolute = sandbox
        .resolve_workspace_relative(&relative)
        .map_err(|e| e.message)?;
    let exists = absolute.exists();

    let if_missing = match delete.if_missing {
        IfMissingStrategy::Fail => IfMissing::Fail,
        IfMissingStrategy::Skip => IfMissing::Skip,
    };

    if !exists && if_missing == IfMissing::Fail {
        return Err(format!("File not found: {relative}"));
    }
    if !exists && if_missing == IfMissing::Skip {
        return Ok(FileChange::Delete {
            path: relative,
            source: String::new(),
            if_missing,
            skipped: true,
        });
    }

    let source = std::fs::read_to_string(&absolute)
        .map_err(|e| format!("Failed to read {relative}: {e}"))?;
    Ok(FileChange::Delete {
        path: relative,
        source,
        if_missing,
        skipped: false,
    })
}

pub fn absolute_paths_for_changes(
    registry: &RecipeRegistry,
    changes: &[FileChange],
) -> Result<Vec<PathBuf>, String> {
    let sandbox = PathSandbox::new(registry.workspace_root.clone());
    changes
        .iter()
        .map(|change| {
            sandbox
                .resolve_workspace_relative(change.path())
                .map_err(|e| e.message)
        })
        .collect()
}

fn relative_codemod_path(registry: &RecipeRegistry) -> String {
    let workspace = registry
        .workspace_root
        .canonicalize()
        .unwrap_or_else(|_| registry.workspace_root.clone());
    let codemod = registry
        .codemod_root()
        .canonicalize()
        .unwrap_or_else(|_| registry.codemod_root().to_path_buf());
    codemod
        .strip_prefix(&workspace)
        .map(|p| p.to_string_lossy().replace('\\', "/"))
        .unwrap_or_else(|_| ".codemod".to_string())
}

fn engine_error_to_string(error: EngineError) -> String {
    error.to_string()
}

pub struct SingleFileRunResult {
    pub file: String,
    pub before: String,
    pub modified: String,
    pub patches: Vec<codemod_recipe_core::patch::SourcePatch>,
}

pub fn run_recipe_on_file(
    registry: &RecipeRegistry,
    recipe_id: &str,
    args: &BTreeMap<String, String>,
) -> Result<SingleFileRunResult, String> {
    let (recipe, recipe_path) = registry.load_recipe_ast(recipe_id)?;
    let collected = collect_recipe_changes(
        registry,
        &recipe,
        Some(recipe_path.as_path()),
        args,
    )?;
    let file = args
        .get("file")
        .cloned()
        .ok_or_else(|| "Missing required arg: file".to_string())?;
    let change = collected
        .changes
        .iter()
        .find(|c| c.path() == file)
        .ok_or_else(|| format!("No changes for file: {file}"))?;
    match change {
        FileChange::Patch { source, patches, .. } => {
            let modified = apply_patches(source, patches).map_err(|e| e.to_string())?;
            Ok(SingleFileRunResult {
                file,
                before: source.clone(),
                modified,
                patches: patches.clone(),
            })
        }
        _ => Err(format!("Expected patch change for file: {file}")),
    }
}

pub fn planned_snapshot_paths(
    registry: &RecipeRegistry,
    recipe: &Recipe,
    args: &BTreeMap<String, String>,
) -> Result<Vec<PathBuf>, String> {
    let merged_maps = registry.merged_maps_for(recipe);
    let effective = crate::args::resolve_effective_args(recipe, args);
    let rendered = render_recipe_templates_with_root(
        recipe,
        &effective,
        &merged_maps,
        registry.vars_by_id(),
        Some(registry.codemod_root()),
    )?;
    let sandbox = PathSandbox::new(registry.workspace_root.clone());
    let mut paths = Vec::new();
    for step in &rendered.steps {
        let relative = match step {
            Step::Edit(edit) => edit.path.clone(),
            Step::Create(create) => create.path.clone(),
            Step::Delete(delete) => delete.path.clone(),
            Step::RecipeRef(_) | Step::Scoped(_) | Step::Unknown(_, _) => continue,
        };
        paths.push(
            sandbox
                .resolve_workspace_relative(&relative)
                .map_err(|e| e.message)?,
        );
    }
    paths.sort();
    paths.dedup();
    Ok(paths)
}
