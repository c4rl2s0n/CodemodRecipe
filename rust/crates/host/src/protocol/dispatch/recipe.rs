//! Preview / apply / diff recipe HostCommand handlers.

use std::collections::BTreeMap;

use codemod_recipe_core::apply::atomic_apply::apply_operations_atomically;

use crate::edit_session::runner::{collect_recipe_changes, planned_snapshot_paths, resolve_recipe};
use crate::lua_runtime::{RecipeApplyError, RunKind};
use crate::preview::diff::build_file_preview_from_change;
use crate::preview::patch_selector::{apply_changes_with_selection, parse_selection};
use crate::preview::preview_token::{compute_preview_token, validate_preview_token};
use crate::protocol::dispatch::common::{to_value, RecipeRequest};
use crate::protocol::{
    logs_from_transcript, ApplyResponse, DiffResponse, DiagnosticSource, PreviewResponse,
    RecipeDiagnostic,
};
use crate::recipe::registry::RecipeRegistry;

pub(super) fn collect(
    registry: &RecipeRegistry,
    request: RecipeRequest<'_>,
    kind: RunKind,
) -> Result<crate::edit_session::runner::CollectedChanges, RecipeApplyError> {
    let (recipe, recipe_path) = resolve_recipe(registry, request.recipe_id, request.inline_recipe)
        .map_err(|message| RecipeApplyError {
            message,
            transcript: Vec::new(),
        })?;
    collect_recipe_changes(
        registry,
        &recipe,
        recipe_path.as_deref(),
        request.args,
        kind,
    )
}

fn preview_fail(
    recipe_key: String,
    error: String,
    source_file: Option<&str>,
    transcript: Option<Vec<codemod_recipe_lua::TranscriptEvent>>,
) -> serde_json::Value {
    let sources = match source_file {
        Some(file) if !file.is_empty() => vec![DiagnosticSource {
            file: file.to_string(),
            line: None,
            column: None,
        }],
        _ => vec![],
    };
    let logs = logs_from_transcript(transcript.as_deref());
    to_value(PreviewResponse {
        ok: false,
        error: Some(error.clone()),
        recipe: Some(recipe_key.clone()),
        preview_token: None,
        files: None,
        transcript,
        logs,
        apply_may_diverge: None,
        diagnostics: Some(vec![RecipeDiagnostic::simple(
            "error",
            "E_RECIPE_RUNTIME",
            error,
            sources,
        )]),
    })
}

pub(super) fn preview(
    registry: &RecipeRegistry,
    request: RecipeRequest<'_>,
    include_contents: bool,
    snippet_lines: Option<u32>,
) -> serde_json::Value {
    let recipe_key = request.recipe_key();
    let recipe_id = request.recipe_id;
    let inline_recipe: Option<&serde_json::Value> = request.inline_recipe;
    let args = request.args.clone();
    let source_file = request
        .recipe_id
        .and_then(|id| registry.get(id))
        .and_then(|r| r.source_file.clone());
    match collect(registry, request, RunKind::Preview) {
        Ok(collected) => {
            let snapshot_paths =
                planned_snapshot_paths(&registry.workspace_root, &collected.changes);
            let path_refs: Vec<_> = snapshot_paths.iter().map(|p| p.as_path()).collect();
            let preview_token = compute_preview_token(recipe_id, inline_recipe, &args, &path_refs);

            let mut files = Vec::new();
            for change in &collected.changes {
                if change.is_skipped() {
                    continue;
                }
                match build_file_preview_from_change(change, include_contents, snippet_lines) {
                    Ok(file) => files.push(file),
                    Err(error) => {
                        return preview_fail(
                            recipe_key,
                            error.to_string(),
                            source_file.as_deref(),
                            Some(collected.transcript.clone()),
                        );
                    }
                }
            }

            let logs = logs_from_transcript(Some(&collected.transcript));
            to_value(PreviewResponse {
                ok: true,
                error: None,
                recipe: Some(recipe_key),
                preview_token: Some(preview_token),
                files: Some(files),
                transcript: Some(collected.transcript),
                logs,
                apply_may_diverge: Some(collected.saw_post_exec),
                diagnostics: None,
            })
        }
        Err(error) => preview_fail(
            recipe_key,
            error.message,
            source_file.as_deref(),
            Some(error.transcript),
        ),
    }
}

pub(super) fn diff(registry: &RecipeRegistry, request: RecipeRequest<'_>, path: &str) -> serde_json::Value {
    let recipe_key = request.recipe_key();
    match collect(registry, request, RunKind::Preview) {
        Ok(collected) => {
            let Some(change) = collected.changes.iter().find(|c| c.path() == path) else {
                return to_value(DiffResponse {
                    ok: false,
                    error: Some(format!("No preview change found for {path}")),
                    recipe: Some(recipe_key),
                    file: None,
                });
            };
            match build_file_preview_from_change(change, true, None) {
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
            error: Some(error.message),
            recipe: Some(recipe_key),
            file: None,
        }),
    }
}

fn apply_fail(
    recipe_key: String,
    error: String,
    source_file: Option<&str>,
    transcript: Option<Vec<codemod_recipe_lua::TranscriptEvent>>,
) -> serde_json::Value {
    let sources = match source_file {
        Some(file) if !file.is_empty() => vec![DiagnosticSource {
            file: file.to_string(),
            line: None,
            column: None,
        }],
        _ => vec![],
    };
    let logs = logs_from_transcript(transcript.as_deref());
    to_value(ApplyResponse {
        ok: false,
        error: Some(error.clone()),
        recipe: Some(recipe_key),
        applied: None,
        transcript,
        logs,
        diagnostics: Some(vec![RecipeDiagnostic::simple(
            "error",
            "E_RECIPE_RUNTIME",
            error,
            sources,
        )]),
    })
}

pub(super) fn apply(
    registry: &RecipeRegistry,
    request: RecipeRequest<'_>,
    preview_token: &str,
    selection: &serde_json::Value,
) -> serde_json::Value {
    let recipe_key = request.recipe_key();
    let recipe_id = request.recipe_id;
    let inline_recipe = request.inline_recipe;
    let args = request.args.clone();
    let source_file = recipe_id
        .and_then(|id| registry.get(id))
        .and_then(|r| r.source_file.clone());

    let preview_collected = match collect(
        registry,
        RecipeRequest {
            recipe_id,
            inline_recipe,
            args: &args,
        },
        RunKind::Preview,
    ) {
        Ok(c) => c,
        Err(error) => {
            return apply_fail(
                recipe_key,
                error.message,
                source_file.as_deref(),
                Some(error.transcript),
            );
        }
    };
    let snapshot_paths =
        planned_snapshot_paths(&registry.workspace_root, &preview_collected.changes);
    let path_refs: Vec<_> = snapshot_paths.iter().map(|p| p.as_path()).collect();
    if let Err(error) =
        validate_preview_token(recipe_id, inline_recipe, &args, preview_token, &path_refs)
    {
        return apply_fail(recipe_key, error, source_file.as_deref(), None);
    }

    let collected = match collect(
        registry,
        RecipeRequest {
            recipe_id,
            inline_recipe,
            args: &args,
        },
        RunKind::Apply,
    ) {
        Ok(c) => c,
        Err(error) => {
            return apply_fail(
                recipe_key,
                error.message,
                source_file.as_deref(),
                Some(error.transcript),
            );
        }
    };

    if collected.saw_post_exec {
        let applied_paths: Vec<String> = collected
            .changes
            .iter()
            .filter(|c| !c.is_skipped())
            .map(|c| c.path().to_string())
            .collect();
        let logs = logs_from_transcript(Some(&collected.transcript));
        return to_value(ApplyResponse {
            ok: true,
            error: None,
            recipe: Some(recipe_key),
            applied: Some(applied_paths),
            transcript: Some(collected.transcript),
            logs,
            diagnostics: None,
        });
    }

    let selection_map = if preview_collected.saw_post_exec {
        BTreeMap::new()
    } else {
        parse_selection(selection)
    };
    let workspace = registry.workspace_root.clone();
    let applied_changes = match apply_changes_with_selection(
        |relative| {
            crate::sandbox::path_sandbox::PathSandbox::new(workspace.clone())
                .resolve_workspace_relative(relative)
                .map_err(|e| e.message)
        },
        &collected.changes,
        &selection_map,
    ) {
        Ok(changes) => changes,
        Err(error) => {
            return apply_fail(
                recipe_key,
                error,
                source_file.as_deref(),
                Some(collected.transcript),
            );
        }
    };

    if applied_changes.is_empty() {
        let logs = logs_from_transcript(Some(&collected.transcript));
        return to_value(ApplyResponse {
            ok: true,
            error: None,
            recipe: Some(recipe_key),
            applied: Some(vec![]),
            transcript: Some(collected.transcript),
            logs,
            diagnostics: None,
        });
    }

    let ops: Vec<_> = applied_changes
        .iter()
        .map(|change| change.operation.clone())
        .collect();
    if let Err(error) = apply_operations_atomically(&ops) {
        return apply_fail(
            recipe_key,
            error,
            source_file.as_deref(),
            Some(collected.transcript),
        );
    }

    let applied_paths: Vec<String> = applied_changes
        .iter()
        .map(|c| c.relative_path.clone())
        .collect();

    let logs = logs_from_transcript(Some(&collected.transcript));
    to_value(ApplyResponse {
        ok: true,
        error: None,
        recipe: Some(recipe_key),
        applied: Some(applied_paths),
        transcript: Some(collected.transcript),
        logs,
        diagnostics: None,
    })
}

