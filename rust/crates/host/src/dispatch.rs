use codemod_recipe_core::atomic_apply::{apply_files_atomically, FileWrite};
use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::diff::build_file_preview;
use crate::patch_selector::{apply_selection, parse_selection};
use crate::post_execution::run_post_execution;
use crate::preview_token::{compute_preview_token, validate_preview_token};
use crate::protocol::{
    ApplyResponse, AstPathResult, DescribeResponse, DiffResponse, HostCommand, PreviewResponse,
    RecipeCatalogResponse, ValidateResponse,
};
use crate::registry::RecipeRegistry;
use crate::runner::{run_recipe_on_file, snapshot_paths_for_args};

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
    })
}

fn validate_response(registry: &RecipeRegistry) -> serde_json::Value {
    let (_, diagnostics) = registry.list();
    let ok = diagnostics.iter().all(|d| d.severity != "error");
    to_value(ValidateResponse {
        ok,
        error: None,
        diagnostics: if diagnostics.is_empty() {
            None
        } else {
            Some(diagnostics)
        },
    })
}

pub fn handle_command(registry: &mut RecipeRegistry, cmd: HostCommand) -> serde_json::Value {
    match cmd {
        HostCommand::Reload => {
            registry.reload();
            catalog_response(registry)
        }
        HostCommand::List => catalog_response(registry),
        HostCommand::Validate => {
            registry.reload();
            validate_response(registry)
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
        HostCommand::Preview {
            recipe,
            args,
            snippet_lines,
        } => preview(registry, &recipe, &args, false, snippet_lines),
        HostCommand::Apply {
            recipe,
            args,
            preview_token,
            selection,
        } => apply(registry, &recipe, &args, &preview_token, &selection),
        HostCommand::Diff { recipe, args, path } => diff(registry, &recipe, &args, &path),
    }
}

fn snapshot_path_refs(registry: &RecipeRegistry, args: &BTreeMap<String, String>) -> Vec<PathBuf> {
    snapshot_paths_for_args(registry, args)
}

fn preview(
    registry: &RecipeRegistry,
    recipe: &str,
    args: &BTreeMap<String, String>,
    include_contents: bool,
    snippet_lines: Option<u32>,
) -> serde_json::Value {
    match run_recipe_on_file(registry, recipe, args) {
        Ok((file, before, result)) => {
            let snapshot_paths = snapshot_path_refs(registry, args);
            let path_refs: Vec<_> = snapshot_paths.iter().map(|p| p.as_path()).collect();
            let preview_token = compute_preview_token(recipe, args, &path_refs);

            if result.patches.is_empty() && before == result.modified {
                return to_value(PreviewResponse {
                    ok: true,
                    error: None,
                    recipe: Some(recipe.to_string()),
                    preview_token: Some(preview_token),
                    files: Some(vec![]),
                });
            }

            let preview_file = build_file_preview(
                file,
                &before,
                &result.modified,
                &result.patches,
                include_contents,
                false,
                snippet_lines,
            );
            to_value(PreviewResponse {
                ok: true,
                error: None,
                recipe: Some(recipe.to_string()),
                preview_token: Some(preview_token),
                files: Some(vec![preview_file]),
            })
        }
        Err(error) => to_value(PreviewResponse {
            ok: false,
            error: Some(error),
            recipe: Some(recipe.to_string()),
            preview_token: None,
            files: None,
        }),
    }
}

fn diff(
    registry: &RecipeRegistry,
    recipe: &str,
    args: &BTreeMap<String, String>,
    path: &str,
) -> serde_json::Value {
    match run_recipe_on_file(registry, recipe, args) {
        Ok((file, before, result)) => {
            if file != path {
                return to_value(DiffResponse {
                    ok: false,
                    error: Some(format!("No preview change found for {path}")),
                    recipe: Some(recipe.to_string()),
                    file: None,
                });
            }
            let preview_file =
                build_file_preview(
                    file,
                    &before,
                    &result.modified,
                    &result.patches,
                    true,
                    true,
                    None,
                );
            to_value(DiffResponse {
                ok: true,
                error: None,
                recipe: Some(recipe.to_string()),
                file: Some(preview_file),
            })
        }
        Err(error) => to_value(DiffResponse {
            ok: false,
            error: Some(error),
            recipe: Some(recipe.to_string()),
            file: None,
        }),
    }
}

fn apply(
    registry: &RecipeRegistry,
    recipe: &str,
    args: &BTreeMap<String, String>,
    preview_token: &str,
    selection: &serde_json::Value,
) -> serde_json::Value {
    let snapshot_paths = snapshot_path_refs(registry, args);
    let path_refs: Vec<_> = snapshot_paths.iter().map(|p| p.as_path()).collect();
    if let Err(error) = validate_preview_token(recipe, args, preview_token, &path_refs) {
        return to_value(ApplyResponse {
            ok: false,
            error: Some(error),
            recipe: Some(recipe.to_string()),
            applied: None,
        });
    }

    match run_recipe_on_file(registry, recipe, args) {
        Ok((file, before, result)) => {
            let selection_map = parse_selection(selection);
            let applied = match apply_selection(&file, &before, &result.patches, &selection_map) {
                Ok(Some((modified, _))) => modified,
                Ok(None) => {
                    return to_value(ApplyResponse {
                        ok: true,
                        error: None,
                        recipe: Some(recipe.to_string()),
                        applied: Some(vec![]),
                    });
                }
                Err(error) => {
                    return to_value(ApplyResponse {
                        ok: false,
                        error: Some(error.to_string()),
                        recipe: Some(recipe.to_string()),
                        applied: None,
                    });
                }
            };

            let file_path = registry.resolve_file_path(&file);
            if let Err(error) = apply_files_atomically(&[FileWrite {
                path: file_path,
                content: applied,
            }]) {
                return to_value(ApplyResponse {
                    ok: false,
                    error: Some(error),
                    recipe: Some(recipe.to_string()),
                    applied: None,
                });
            }

            if let Ok((recipe_ast, _)) = registry.load_recipe_ast(recipe) {
                if let Err(error) =
                    run_post_execution(&recipe_ast.post_execution, args, std::slice::from_ref(&file))
                {
                    return to_value(ApplyResponse {
                        ok: false,
                        error: Some(error),
                        recipe: Some(recipe.to_string()),
                        applied: None,
                    });
                }
            }

            to_value(ApplyResponse {
                ok: true,
                error: None,
                recipe: Some(recipe.to_string()),
                applied: Some(vec![file]),
            })
        }
        Err(error) => to_value(ApplyResponse {
            ok: false,
            error: Some(error),
            recipe: Some(recipe.to_string()),
            applied: None,
        }),
    }
}

fn to_value<T: serde::Serialize>(value: T) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or_else(
        |e| serde_json::json!({ "ok": false, "error": format!("serialization failed: {e}") }),
    )
}
