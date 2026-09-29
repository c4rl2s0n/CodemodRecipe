use codemod_recipe_core::apply::atomic_apply::apply_operations_atomically;
use std::collections::BTreeMap;

use crate::component::{
    component_coverage, component_incremental_scan, component_scan, invoke_hook, invoke_operation,
    invoke_scaffold, ComponentHookRequest, ComponentIncrementalScanRequest, ComponentInvokeRequest,
    ComponentScaffoldRequest,
};
use crate::engine::runner::{collect_recipe_changes, planned_snapshot_paths, resolve_recipe};
use crate::lua_runtime::{RecipeApplyError, RunKind};
use crate::preview::diff::build_file_preview_from_change;
use crate::preview::patch_selector::{apply_changes_with_selection, parse_selection};
use crate::preview::preview_token::{compute_preview_token, validate_preview_token};
use crate::protocol::keys as protocol_keys;
use crate::protocol::{
    ApplyResponse, AstPathResult, ComponentGraphResponse, DescribeComponentResponse,
    DescribeResponse, DiagnosticSource, DiffResponse, HostCommand, PreviewResponse,
    QueryCapturesResponse, RecipeCatalogResponse, RecipeDiagnostic,
};
use crate::recipe::registry::RecipeRegistry;
use crate::recipe::validate::{validate_recipe, validate_workspace};

struct RecipeRequest<'a> {
    recipe_id: Option<&'a str>,
    inline_recipe: Option<&'a serde_json::Value>,
    args: &'a BTreeMap<String, serde_json::Value>,
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
    })
}

fn extract_capture_names(query_source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let bytes = query_source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            i += 1;
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            if i > start {
                let name = &query_source[start..i];
                if seen.insert(name.to_string()) {
                    names.push(name.to_string());
                }
            }
            continue;
        }
        i += 1;
    }
    names
}

fn query_captures_response(
    registry: &RecipeRegistry,
    query: Option<&str>,
    query_id: Option<&str>,
) -> QueryCapturesResponse {
    let source = if let Some(id) = query_id {
        match registry.queries_by_id().get(id) {
            Some(s) => s.clone(),
            None => {
                return QueryCapturesResponse {
                    ok: false,
                    error: Some(format!("unknown query id '{id}'")),
                    captures: None,
                };
            }
        }
    } else if let Some(q) = query {
        q.to_string()
    } else {
        return QueryCapturesResponse {
            ok: false,
            error: Some("query or queryId is required".to_string()),
            captures: None,
        };
    };
    QueryCapturesResponse {
        ok: true,
        error: None,
        captures: Some(extract_capture_names(&source)),
    }
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
        HostCommand::GenerateAstPath { path, offset } => to_value(AstPathResult {
            ok: false,
            error: Some(format!(
                "generateAstPath is not supported by the Rust host (v1) (path={path}, offset={offset})"
            )),
        }),
        HostCommand::Bootstrap {
            force,
            edit_policy,
            companions,
        } => {
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
        HostCommand::DeriveArgs {
            recipe,
            source,
            language,
            path,
            cursor_offset,
            selection_start,
            selection_end,
            context,
        } => to_value(crate::recipe::derive_args::derive_args(
            registry,
            crate::recipe::derive_args::DeriveArgsRequest {
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
            crate::explorer::filter_explorer_recipes::filter_explorer_recipes(
                registry, &path, &kind,
            ),
        ),
        HostCommand::DumpAst {
            path,
            source,
            language,
            named_only,
        } => to_value(crate::query::query_tools::handle_dump_ast(
            registry,
            path.as_deref(),
            source.as_deref(),
            language.as_deref(),
            named_only,
        )),
        HostCommand::DebugQuery {
            path,
            source,
            language,
            query,
            instrument,
            include_sexp,
        } => to_value(crate::query::query_tools::handle_debug_query(
            registry,
            path.as_deref(),
            source.as_deref(),
            language.as_deref(),
            &query,
            instrument,
            include_sexp,
        )),
        HostCommand::GenerateQuery {
            path,
            source,
            language,
            start,
            end,
            include_text_predicates,
            capture_leaf,
            max_depth,
        } => to_value(crate::query::query_tools::handle_generate_query(
            registry,
            path.as_deref(),
            source.as_deref(),
            language.as_deref(),
            start,
            end,
            include_text_predicates,
            capture_leaf.as_deref(),
            max_depth,
        )),
        HostCommand::ResolveStaticPath { template } => to_value(
            crate::query::query_tools::handle_resolve_static_path(&template),
        ),
        HostCommand::QueryCaptures { query, query_id } => to_value(query_captures_response(
            registry,
            query.as_deref(),
            query_id.as_deref(),
        )),
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
        HostCommand::ComponentScan => to_value(component_scan(registry)),
        HostCommand::ComponentIncrementalScan {
            dirty_paths,
            detect_changes,
            force,
        } => to_value(component_incremental_scan(
            registry,
            ComponentIncrementalScanRequest {
                dirty_paths,
                detect_changes,
                force,
            },
        )),
        HostCommand::ComponentListInstances => {
            let scan = component_scan(registry);
            to_value(serde_json::json!({
                "ok": scan.ok,
                "error": scan.error,
                "instances": scan.instances,
                "findings": scan.findings,
                "diagnostics": scan.diagnostics,
            }))
        }
        HostCommand::ComponentCoverage => to_value(component_coverage(registry)),
        HostCommand::ComponentGraph { level } => {
            let level = level.as_deref().unwrap_or("kinds");
            if level != "kinds" && level != "instances" {
                to_value(ComponentGraphResponse {
                    ok: false,
                    error: Some(format!(
                        "unknown componentGraph level '{level}' (expected 'kinds' or 'instances')"
                    )),
                    mermaid: None,
                    kind_graph: None,
                    edges: None,
                })
            } else {
                // Scan-on-demand: component_scan populates the cache when empty.
                let scan = component_scan(registry);
                if !scan.ok {
                    to_value(ComponentGraphResponse {
                        ok: false,
                        error: Some(
                            scan.error
                                .unwrap_or_else(|| "component scan failed".to_string()),
                        ),
                        mermaid: None,
                        kind_graph: None,
                        edges: None,
                    })
                } else {
                    let kind_graph = scan.kind_graph.clone().unwrap_or_default();
                    let edges = scan.edges.clone().unwrap_or_default();
                    let (mermaid, out_edges) = if level == "instances" {
                        let instances = scan.instances.unwrap_or_default();
                        let flat = crate::component::relations::flatten_instances(&instances);
                        (
                            crate::component::render_instance_graph_mermaid(&edges, &flat),
                            Some(edges),
                        )
                    } else {
                        (crate::component::render_kind_graph_mermaid(&kind_graph), None)
                    };
                    to_value(ComponentGraphResponse {
                        ok: true,
                        error: None,
                        mermaid: Some(mermaid),
                        kind_graph: Some(kind_graph),
                        edges: out_edges,
                    })
                }
            }
        }
        HostCommand::ComponentInvokeOperation {
            instance_id,
            operation,
            args,
            preview,
        } => to_value(invoke_operation(
            registry,
            ComponentInvokeRequest {
                instance_id,
                operation,
                args,
                kind: if preview {
                    RunKind::Preview
                } else {
                    RunKind::Apply
                },
            },
        )),
        HostCommand::ComponentScaffold {
            component_id,
            args,
            preview,
        } => to_value(invoke_scaffold(
            registry,
            ComponentScaffoldRequest {
                component_id,
                args,
                kind: if preview {
                    RunKind::Preview
                } else {
                    RunKind::Apply
                },
            },
        )),
        HostCommand::ComponentInvokeHook {
            component_id,
            hook,
            preview,
            locator,
            instance,
            operation,
            args,
        } => to_value(invoke_hook(
            registry,
            ComponentHookRequest {
                component_id,
                hook,
                kind: if preview {
                    RunKind::Preview
                } else {
                    RunKind::Apply
                },
                locator,
                instance,
                operation,
                args,
            },
        )),
        HostCommand::DescribeComponent { component_id } => {
            match registry.describe_component(&component_id) {
                Some(component) => to_value(DescribeComponentResponse {
                    ok: true,
                    error: None,
                    component: Some(component),
                }),
                None => to_value(DescribeComponentResponse {
                    ok: false,
                    error: Some(format!("component not found: {component_id}")),
                    component: None,
                }),
            }
        }
        HostCommand::RunTests { filter } => to_value(crate::testing::run_tests(
            registry,
            filter.as_deref(),
        )),
    }
}

fn collect(
    registry: &RecipeRegistry,
    request: RecipeRequest<'_>,
    kind: RunKind,
) -> Result<crate::engine::runner::CollectedChanges, RecipeApplyError> {
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
    to_value(PreviewResponse {
        ok: false,
        error: Some(error.clone()),
        recipe: Some(recipe_key.clone()),
        preview_token: None,
        files: None,
        transcript,
        apply_may_diverge: None,
        diagnostics: Some(vec![RecipeDiagnostic::simple(
            "error",
            "E_RECIPE_RUNTIME",
            error,
            sources,
        )]),
    })
}

fn preview(
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

            to_value(PreviewResponse {
                ok: true,
                error: None,
                recipe: Some(recipe_key),
                preview_token: Some(preview_token),
                files: Some(files),
                transcript: Some(collected.transcript),
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

fn diff(registry: &RecipeRegistry, request: RecipeRequest<'_>, path: &str) -> serde_json::Value {
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
    to_value(ApplyResponse {
        ok: false,
        error: Some(error.clone()),
        recipe: Some(recipe_key),
        applied: None,
        transcript,
        diagnostics: Some(vec![RecipeDiagnostic::simple(
            "error",
            "E_RECIPE_RUNTIME",
            error,
            sources,
        )]),
    })
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
        return to_value(ApplyResponse {
            ok: true,
            error: None,
            recipe: Some(recipe_key),
            applied: Some(applied_paths),
            transcript: Some(collected.transcript),
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
        return to_value(ApplyResponse {
            ok: true,
            error: None,
            recipe: Some(recipe_key),
            applied: Some(vec![]),
            transcript: Some(collected.transcript),
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

    to_value(ApplyResponse {
        ok: true,
        error: None,
        recipe: Some(recipe_key),
        applied: Some(applied_paths),
        transcript: Some(collected.transcript),
        diagnostics: None,
    })
}

fn to_value<T: serde::Serialize>(value: T) -> serde_json::Value {
    serde_json::to_value(value).unwrap_or_else(|e| error_json(format!("serialization failed: {e}")))
}

#[cfg(test)]
mod query_captures_tests {
    use super::extract_capture_names;

    #[test]
    fn extracts_unique_captures_in_order() {
        let q = r#"
(class_definition
  name: (identifier) @className
  body: (class_body (block) @body)
  (#eq? @className "X"))
"#;
        assert_eq!(
            extract_capture_names(q),
            vec!["className".to_string(), "body".to_string()]
        );
    }
}

#[test]
fn test_handle_preview() {
    let mut registry = RecipeRegistry::new(
        "/home/ikusa/workspace/rust/domtree".into(),
        "/home/ikusa/workspace/rust/domtree/.codemod".into(),
    );
    registry.reload();
    let json = handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("test".to_string()),
            inline_recipe: None,
            args: BTreeMap::new(),
            snippet_lines: None,
        },
    );
    let jstring = json.to_string();
    println!("{jstring}");
}
