//! Derive recipe args from editor buffer context (builtins, templates, queries).

use std::collections::BTreeMap;
use std::path::Path;

use codemod_recipe_engine::engine::{CaptureSpan, Engine, EngineError, QueryContext};
use codemod_recipe_engine::LanguageRegistry;
use codemod_recipe_yaml::arg_from::{ArgFrom, ArgFromOnNoMatch, ArgFromScope, ArgFromSpec};
use codemod_recipe_yaml::let_binding::{LetBinding, LetExtract, LetOnManyMatches, LetOnNoMatch};
use codemod_recipe_yaml::model::{Arg, Recipe};
use codemod_recipe_yaml::QuerySpec;

use crate::protocol::DeriveArgsResponse;
use crate::registry::{render_query_op_public, RecipeRegistry};
use crate::render_context::RecipeRenderContext;
use crate::template::render_template;

pub struct DeriveArgsRequest<'a> {
    pub recipe_id: &'a str,
    pub source: &'a str,
    pub language: Option<&'a str>,
    pub path: Option<&'a str>,
    pub cursor_offset: usize,
    pub selection_start: usize,
    pub selection_end: usize,
    pub context: BTreeMap<String, String>,
}

pub fn derive_args(registry: &RecipeRegistry, req: DeriveArgsRequest<'_>) -> DeriveArgsResponse {
    let (recipe, recipe_path) = match registry.load_recipe_ast(req.recipe_id) {
        Ok(v) => v,
        Err(err) => {
            return DeriveArgsResponse {
                ok: false,
                error: Some(err),
                args: None,
            };
        }
    };

    let mut language_registry = LanguageRegistry::with_config(registry.language_config.clone());
    let maps = registry.merged_maps_for(&recipe);
    let vars = registry.vars_by_id().clone();

    let mut derived = BTreeMap::new();
    let mut merge_ctx = req.context.clone();

    for arg in &recipe.args {
        match resolve_arg_from(
            registry,
            &recipe,
            recipe_path.as_path(),
            &mut language_registry,
            &maps,
            &vars,
            arg,
            &req,
            &merge_ctx,
        ) {
            Ok(Some(value)) if !value.is_empty() => {
                derived.insert(arg.name.clone(), value.clone());
                merge_ctx.insert(arg.name.clone(), value);
            }
            Ok(Some(_)) => {
                // empty with onNoMatch: empty — leave unset for omit default
            }
            Ok(None) => {}
            Err(_) => {}
        }
    }

    DeriveArgsResponse {
        ok: true,
        error: None,
        args: Some(derived),
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_arg_from(
    registry: &RecipeRegistry,
    recipe: &Recipe,
    recipe_path: &Path,
    languages: &mut LanguageRegistry,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    arg: &Arg,
    req: &DeriveArgsRequest<'_>,
    merge_ctx: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let from = match (&arg.from, &arg.context_key) {
        (Some(f), _) => f.clone(),
        (None, Some(key)) => ArgFrom::Builtin(key.clone()),
        (None, None) => return Ok(None),
    };

    match from {
        ArgFrom::Builtin(key) => Ok(merge_ctx.get(&key).filter(|v| !v.is_empty()).cloned()),
        ArgFrom::Spec(spec) => resolve_spec(
            registry,
            recipe,
            recipe_path,
            languages,
            maps,
            vars,
            &arg.name,
            &spec,
            req,
            merge_ctx,
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn resolve_spec(
    registry: &RecipeRegistry,
    recipe: &Recipe,
    recipe_path: &Path,
    languages: &mut LanguageRegistry,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    arg_name: &str,
    spec: &ArgFromSpec,
    req: &DeriveArgsRequest<'_>,
    merge_ctx: &BTreeMap<String, String>,
) -> Result<Option<String>, String> {
    let extracted = if let Some(query) = &spec.query {
        Some(evaluate_query_from(
            registry,
            recipe,
            recipe_path,
            languages,
            maps,
            vars,
            spec,
            query,
            req,
            merge_ctx,
        )?)
    } else {
        None
    };

    let final_value = if let Some(as_tmpl) = &spec.r#as {
        let mut ctx = merge_ctx.clone();
        if let Some(v) = &extracted {
            ctx.insert(arg_name.to_string(), v.clone());
            ctx.insert("_".to_string(), v.clone());
            if let Some(cap) = &spec.capture {
                ctx.insert(cap.clone(), v.clone());
            }
        }
        Some(render_template(as_tmpl, &ctx, maps, vars)?)
    } else if let Some(tmpl) = &spec.template {
        Some(render_template(tmpl, merge_ctx, maps, vars)?)
    } else {
        extracted
    };

    match final_value {
        Some(v) if v.is_empty() => match spec.on_no_match.unwrap_or_default() {
            ArgFromOnNoMatch::Empty => Ok(Some(String::new())),
            ArgFromOnNoMatch::Omit => Ok(None),
        },
        other => Ok(other),
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_query_from(
    registry: &RecipeRegistry,
    recipe: &Recipe,
    recipe_path: &Path,
    languages: &mut LanguageRegistry,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    spec: &ArgFromSpec,
    query: &QuerySpec,
    req: &DeriveArgsRequest<'_>,
    merge_ctx: &BTreeMap<String, String>,
) -> Result<String, String> {
    let file_path = req.path.unwrap_or("buffer");
    let explicit_lang = spec
        .language
        .as_deref()
        .or(req.language)
        .filter(|s| !s.is_empty());

    // Map common VS Code language ids onto tree-sitter language pack ids.
    let mapped = explicit_lang.map(map_editor_language_id);
    let engine = languages
        .resolve_for_edit(mapped.as_deref(), file_path)
        .map_err(|e| e.to_string())?;

    let render = RecipeRenderContext::with_registry(
        recipe,
        registry,
        Some(recipe_path),
        merge_ctx,
        maps,
        vars,
    );
    let rendered_query = render_query_op_public(query, &render, merge_ctx)?;
    let capture = if let Some(cap) = &spec.capture {
        Some(render_template(cap, merge_ctx, maps, vars)?)
    } else {
        None
    };

    let binding = LetBinding {
        name: "derived".to_string(),
        query: Some(rendered_query),
        capture: capture.clone(),
        extract: spec.extract.unwrap_or(LetExtract::Text),
        on_no_match: match spec.on_no_match.unwrap_or_default() {
            ArgFromOnNoMatch::Omit | ArgFromOnNoMatch::Empty => LetOnNoMatch::UseEmpty,
        },
        on_many_matches: spec.on_many_matches.unwrap_or(LetOnManyMatches::First),
        join: spec.join.clone(),
        r#as: None,
    };

    let ctx = QueryContext {
        recipe_file: Some(recipe_path),
        codemod_root: registry.codemod_root(),
    };

    let scope = spec.scope.unwrap_or(ArgFromScope::Enclosing);
    evaluate_with_scope(
        engine,
        &ctx,
        req.source,
        &binding,
        scope,
        req.cursor_offset,
        req.selection_start,
        req.selection_end,
    )
}

fn map_editor_language_id(id: &str) -> String {
    match id {
        "typescriptreact" => "tsx".to_string(),
        "javascriptreact" => "jsx".to_string(),
        "csharp" => "c_sharp".to_string(),
        other => other.to_string(),
    }
}

#[allow(clippy::too_many_arguments)] // scope + selection span mirrors editor protocol
fn evaluate_with_scope(
    engine: &mut Engine,
    ctx: &QueryContext<'_>,
    source: &str,
    binding: &LetBinding,
    scope: ArgFromScope,
    cursor: usize,
    sel_start: usize,
    sel_end: usize,
) -> Result<String, String> {
    match scope {
        ArgFromScope::First => engine
            .evaluate_let_binding(ctx, source, binding)
            .map_err(engine_err),
        ArgFromScope::Enclosing | ArgFromScope::Selection => {
            let capture = binding.capture.as_deref().ok_or_else(|| {
                "from.query with enclosing/selection scope requires capture".to_string()
            })?;
            let query = binding
                .query
                .as_ref()
                .ok_or_else(|| "from.query required".to_string())?;
            let spans = engine
                .query_capture_spans(ctx, source, query, capture)
                .map_err(engine_err)?;
            let filtered = filter_spans(&spans, scope, cursor, sel_start, sel_end);
            if filtered.is_empty() {
                return match binding.on_no_match {
                    LetOnNoMatch::UseEmpty => Ok(String::new()),
                    LetOnNoMatch::Error => Err(format!("no match for capture '{capture}'")),
                };
            }
            let chosen = &filtered[0];
            match binding.extract {
                LetExtract::Text | LetExtract::Exists | LetExtract::Count => Ok(source
                    .get(chosen.start..chosen.end)
                    .unwrap_or("")
                    .to_string()),
                LetExtract::Kind => {
                    // Fall back to first-match let eval with First policy.
                    let mut first_binding = binding.clone();
                    first_binding.on_many_matches = LetOnManyMatches::First;
                    engine
                        .evaluate_let_binding(ctx, source, &first_binding)
                        .map_err(engine_err)
                }
            }
        }
    }
}

fn filter_spans(
    spans: &[CaptureSpan],
    scope: ArgFromScope,
    cursor: usize,
    sel_start: usize,
    sel_end: usize,
) -> Vec<CaptureSpan> {
    match scope {
        ArgFromScope::First => spans.to_vec(),
        ArgFromScope::Enclosing => {
            let mut enclosing: Vec<CaptureSpan> = spans
                .iter()
                .copied()
                .filter(|s| s.start <= cursor && cursor <= s.end)
                .collect();
            enclosing.sort_by_key(|s| s.end.saturating_sub(s.start));
            enclosing
        }
        ArgFromScope::Selection => {
            let (a, b) = if sel_start <= sel_end {
                (sel_start, sel_end)
            } else {
                (sel_end, sel_start)
            };
            spans
                .iter()
                .copied()
                .filter(|s| s.start < b && s.end > a)
                .collect()
        }
    }
}

fn engine_err(err: EngineError) -> String {
    err.to_string()
}
