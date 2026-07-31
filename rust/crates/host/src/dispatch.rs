use codemod_recipe_core::atomic_apply::apply_operations_atomically;
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::diff::build_file_preview_from_change;
use crate::patch_selector::{apply_changes_with_selection, parse_selection};
use crate::post_execution::run_post_execution;
use crate::preview_token::{compute_preview_token, validate_preview_token};
use crate::protocol::{
    ApplyResponse, AstPathResult, DescribeResponse, DiffResponse, HostCommand, PreviewResponse,
    RecipeCatalogResponse,
};
use crate::protocol_keys;
use crate::registry::RecipeRegistry;
use crate::runner::{collect_recipe_changes, planned_snapshot_paths, resolve_recipe};
use crate::validate::{validate_recipe, validate_workspace};

struct RecipeRequest<'a> {
    recipe_id: Option<&'a str>,
    inline_recipe: Option<&'a serde_json::Value>,
    args: &'a BTreeMap<String, String>,
}

impl<'a> RecipeRequest<'a> {
    fn recipe_key(&self) -> String {
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

fn error_json(message: impl Into<String>) -> serde_json::Value {
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

fn catalog_response(registry: &RecipeRegistry) -> serde_json::Value {
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
        language_ids: Some(
            codemod_recipe_engine::native::native_language_ids()
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
        ),
    })
}

pub fn handle_command(registry: &mut RecipeRegistry, cmd: HostCommand) -> serde_json::Value {
    match cmd {
        HostCommand::Reload => {
            registry.reload();
            catalog_response(registry)
        }
        HostCommand::List => catalog_response(registry),
        HostCommand::Validate { recipe } => {
            if let Some(recipe_id) = recipe {
                to_value(validate_recipe(registry, &recipe_id))
            } else {
                to_value(validate_workspace(registry))
            }
        }
        HostCommand::Describe { recipe } => match registry.get(&recipe) {
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
        },
        HostCommand::GenerateAstPath { .. } => to_value(AstPathResult {
            ok: false,
            error: Some("generateAstPath is not supported by the Rust host (v1)".to_string()),
        }),
        HostCommand::Bootstrap {
            force,
            edit_policy,
            companions,
        } => {
            let policy = match crate::bootstrap::EditPolicy::parse(
                edit_policy.as_deref().unwrap_or("recommend"),
            ) {
                Ok(p) => p,
                Err(e) => return error_json(e),
            };
            let companions = match crate::bootstrap::parse_companions(&companions) {
                Ok(c) => c,
                Err(e) => return error_json(e),
            };
            crate::bootstrap::bootstrap_project(
                &registry.workspace_root,
                force,
                policy,
                &companions,
            )
        }
        HostCommand::DeriveArgs {
            recipe,
            source,
            language,
            path,
            cursor_offset,
            selection_start,
            selection_end,
            context,
        } => to_value(crate::derive_args::derive_args(
            registry,
            crate::derive_args::DeriveArgsRequest {
                recipe_id: &recipe,
                source: &source,
                language: language.as_deref(),
                path: path.as_deref(),
                cursor_offset: cursor_offset as usize,
                selection_start: selection_start as usize,
                selection_end: selection_end as usize,
                context,
            },
        )),
        HostCommand::FilterExplorerRecipes { path, kind } => to_value(
            crate::filter_explorer_recipes::filter_explorer_recipes(registry, &path, &kind),
        ),
        HostCommand::Preview {
            recipe,
            inline_recipe,
            args,
            snippet_lines,
        } => preview(
            registry,
            RecipeRequest {
                recipe_id: recipe.as_deref(),
                inline_recipe: inline_recipe.as_ref(),
                args: &args,
            },
            true,
            snippet_lines,
        ),
        HostCommand::Apply {
            recipe,
            inline_recipe,
            args,
            preview_token,
            selection,
        } => apply(
            registry,
            RecipeRequest {
                recipe_id: recipe.as_deref(),
                inline_recipe: inline_recipe.as_ref(),
                args: &args,
            },
            &preview_token,
            &selection,
        ),
        HostCommand::Diff {
            recipe,
            inline_recipe,
            args,
            path,
        } => diff(
            registry,
            RecipeRequest {
                recipe_id: recipe.as_deref(),
                inline_recipe: inline_recipe.as_ref(),
                args: &args,
            },
            &path,
        ),
    }
}

fn collect(
    registry: &RecipeRegistry,
    request: RecipeRequest<'_>,
) -> Result<crate::runner::CollectedChanges, String> {
    let (recipe, recipe_path) = resolve_recipe(registry, request.recipe_id, request.inline_recipe)?;
    collect_recipe_changes(registry, &recipe, recipe_path.as_deref(), request.args)
}

fn snapshot_paths_for_request(
    registry: &RecipeRegistry,
    request: RecipeRequest<'_>,
) -> Result<Vec<PathBuf>, String> {
    let (recipe, _) = resolve_recipe(registry, request.recipe_id, request.inline_recipe)?;
    planned_snapshot_paths(registry, &recipe, request.args)
}

fn preview(
    registry: &RecipeRegistry,
    request: RecipeRequest<'_>,
    include_contents: bool,
    snippet_lines: Option<u32>,
) -> serde_json::Value {
    let recipe_key = request.recipe_key();
    let recipe_id = request.recipe_id;
    let inline_recipe = request.inline_recipe;
    let args = request.args.clone();
    match collect(registry, request) {
        Ok(collected) => {
            let snapshot_paths = match snapshot_paths_for_request(
                registry,
                RecipeRequest {
                    recipe_id,
                    inline_recipe,
                    args: &args,
                },
            ) {
                Ok(paths) => paths,
                Err(error) => {
                    return to_value(PreviewResponse {
                        ok: false,
                        error: Some(error),
                        recipe: Some(recipe_key),
                        preview_token: None,
                        files: None,
                    });
                }
            };
            let path_refs: Vec<_> = snapshot_paths.iter().map(|p| p.as_path()).collect();
            let preview_token = compute_preview_token(recipe_id, inline_recipe, &args, &path_refs);

            let mut files = Vec::new();
            for change in &collected.changes {
                if change.is_skipped() {
                    continue;
                }
                match build_file_preview_from_change(change, include_contents, false, snippet_lines)
                {
                    Ok(file) => files.push(file),
                    Err(error) => {
                        return to_value(PreviewResponse {
                            ok: false,
                            error: Some(error.to_string()),
                            recipe: Some(recipe_key),
                            preview_token: None,
                            files: None,
                        });
                    }
                }
            }

            to_value(PreviewResponse {
                ok: true,
                error: None,
                recipe: Some(recipe_key),
                preview_token: Some(preview_token),
                files: Some(files),
            })
        }
        Err(error) => to_value(PreviewResponse {
            ok: false,
            error: Some(error),
            recipe: Some(recipe_key),
            preview_token: None,
            files: None,
        }),
    }
}

fn diff(registry: &RecipeRegistry, request: RecipeRequest<'_>, path: &str) -> serde_json::Value {
    let recipe_key = request.recipe_key();
    match collect(registry, request) {
        Ok(collected) => {
            let Some(change) = collected.changes.iter().find(|c| c.path() == path) else {
                return to_value(DiffResponse {
                    ok: false,
                    error: Some(format!("No preview change found for {path}")),
                    recipe: Some(recipe_key),
                    file: None,
                });
            };
            match build_file_preview_from_change(change, true, true, None) {
                Ok(file) => to_value(DiffResponse {
                    ok: true,
                    error: None,
                    recipe: Some(recipe_key),
                    file: Some(file),
                }),
                Err(error) => to_value(DiffResponse {
                    ok: false,
                    error: Some(error.to_string()),
                    recipe: Some(recipe_key),
                    file: None,
                }),
            }
        }
        Err(error) => to_value(DiffResponse {
            ok: false,
            error: Some(error),
            recipe: Some(recipe_key),
            file: None,
        }),
    }
}

fn apply(
    registry: &RecipeRegistry,
    request: RecipeRequest<'_>,
    preview_token: &str,
    selection: &serde_json::Value,
) -> serde_json::Value {
    let recipe_key = request.recipe_key();
    let recipe_id = request.recipe_id;
    let inline_recipe = request.inline_recipe;
    let args = request.args.clone();

    let snapshot_paths = match snapshot_paths_for_request(
        registry,
        RecipeRequest {
            recipe_id,
            inline_recipe,
            args: &args,
        },
    ) {
        Ok(paths) => paths,
        Err(error) => {
            return to_value(ApplyResponse {
                ok: false,
                error: Some(error),
                recipe: Some(recipe_key),
                applied: None,
            });
        }
    };
    let path_refs: Vec<_> = snapshot_paths.iter().map(|p| p.as_path()).collect();
    if let Err(error) =
        validate_preview_token(recipe_id, inline_recipe, &args, preview_token, &path_refs)
    {
        return to_value(ApplyResponse {
            ok: false,
            error: Some(error),
            recipe: Some(recipe_key),
            applied: None,
        });
    }

    let collected = match collect(
        registry,
        RecipeRequest {
            recipe_id,
            inline_recipe,
            args: &args,
        },
    ) {
        Ok(c) => c,
        Err(error) => {
            return to_value(ApplyResponse {
                ok: false,
                error: Some(error),
                recipe: Some(recipe_key),
                applied: None,
            });
        }
    };

    let selection_map = parse_selection(selection);
    let workspace = registry.workspace_root.clone();
    let applied_changes = match apply_changes_with_selection(
        |relative| {
            crate::path_sandbox::PathSandbox::new(workspace.clone())
                .resolve_workspace_relative(relative)
                .map_err(|e| e.message)
        },
        &collected.changes,
        &selection_map,
    ) {
        Ok(changes) => changes,
        Err(error) => {
            return to_value(ApplyResponse {
                ok: false,
                error: Some(error),
                recipe: Some(recipe_key),
                applied: None,
            });
        }
    };

    if applied_changes.is_empty() {
        return to_value(ApplyResponse {
            ok: true,
            error: None,
            recipe: Some(recipe_key),
            applied: Some(vec![]),
        });
    }

    let ops: Vec<_> = applied_changes
        .iter()
        .map(|change| change.operation.clone())
        .collect();
    if let Err(error) = apply_operations_atomically(&ops) {
        return to_value(ApplyResponse {
            ok: false,
            error: Some(error),
            recipe: Some(recipe_key),
            applied: None,
        });
    }

    let applied_paths: Vec<String> = applied_changes
        .iter()
        .map(|c| c.relative_path.clone())
        .collect();

    if let Err(error) = run_post_execution(
        &collected.recipe.post_execution,
        &args,
        registry.maps_by_id(),
        registry.vars_by_id(),
        &registry.workspace_root,
        registry.codemod_root(),
        collected.recipe_path.as_deref(),
    ) {
        return to_value(ApplyResponse {
            ok: false,
            error: Some(error),
            recipe: Some(recipe_key),
            applied: None,
        });
    }

    to_value(ApplyResponse {
        ok: true,
        error: None,
        recipe: Some(recipe_key),
        applied: Some(applied_paths),
    })
}

fn to_value<T: serde::Serialize>(value: T) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or_else(|e| error_json(format!("serialization failed: {e}")))
}
