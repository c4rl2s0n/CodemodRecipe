//! Apply an edit step with `when` / `whenNot` guards and per-op `let` bindings.

use std::collections::BTreeMap;

use codemod_recipe_engine::engine::{Engine, EngineError, QueryContext};
use codemod_recipe_yaml::let_binding::LetBinding;
use codemod_recipe_yaml::model::{EditOp, EditStep};
use codemod_recipe_yaml::GuardList;

use crate::registry::{render_query_op_public, RecipeRegistry};
use crate::render_context::RecipeRenderContext;
use crate::template::render_template;

/// Returns `Ok(None)` when guards skip the edit (no error).
pub fn apply_edit_step_with_guards(
    engine: &mut Engine,
    query_ctx: &QueryContext<'_>,
    edit: &EditStep,
    render: &RecipeRenderContext<'_>,
    source: &str,
) -> Result<Option<String>, String> {
    if !guards_pass(
        engine,
        query_ctx,
        source,
        edit.when.as_ref(),
        edit.when_not.as_ref(),
    )? {
        return Ok(None);
    }

    let mut current = source.to_string();
    for op in &edit.ops {
        let locals = evaluate_let_bindings(engine, query_ctx, &current, &edit.let_bindings, render)?;
        let rendered_op = render_single_edit_op(op, render, &locals)?;
        let patches = engine
            .collect_patches_for_single_op(query_ctx, &rendered_op, &current)
            .map_err(engine_error_to_string)?;
        if !patches.is_empty() {
            current = codemod_recipe_core::patch::apply_patches(&current, &patches)
                .map_err(|e| e.to_string())?;
        }
    }
    if current == source {
        Ok(None)
    } else {
        Ok(Some(current))
    }
}

fn guards_pass(
    engine: &mut Engine,
    ctx: &QueryContext<'_>,
    source: &str,
    when: Option<&GuardList>,
    when_not: Option<&GuardList>,
) -> Result<bool, String> {
    if let Some(when_list) = when {
        for spec in &when_list.guards {
            if !engine
                .query_has_match(ctx, source, spec)
                .map_err(engine_error_to_string)?
            {
                return Ok(false);
            }
        }
    }
    if let Some(when_not_list) = when_not {
        for spec in &when_not_list.guards {
            if engine
                .query_has_match(ctx, source, spec)
                .map_err(engine_error_to_string)?
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn evaluate_let_bindings(
    engine: &mut Engine,
    ctx: &QueryContext<'_>,
    source: &str,
    bindings: &codemod_recipe_yaml::LetBindings,
    render: &RecipeRenderContext<'_>,
) -> Result<BTreeMap<String, String>, String> {
    let registry = render
        .registry
        .ok_or_else(|| "recipe registry required for let bindings".to_string())?;
    let mut locals = BTreeMap::new();
    let mut context = render.args.clone();
    for binding in &bindings.0 {
        let extracted = if binding.query.is_some() {
            let rendered_binding =
                render_let_binding_for_eval(binding, render, registry, &context)?;
            Some(
                engine
                    .evaluate_let_binding(ctx, source, &rendered_binding)
                    .map_err(engine_error_to_string)?,
            )
        } else if binding.r#as.is_some() {
            None
        } else {
            return Err(format!(
                "let binding '{}' requires query or as",
                binding.name
            ));
        };
        let mut merge_ctx = merge_args_and_locals(render.args, &locals);
        if let Some(v) = &extracted {
            merge_ctx.insert(binding.name.clone(), v.clone());
        }
        let final_value = if let Some(as_tmpl) = &binding.r#as {
            render_template(as_tmpl, &merge_ctx, render.maps, render.vars)?
        } else {
            extracted.ok_or_else(|| format!("let binding '{}' missing query", binding.name))?
        };
        locals.insert(binding.name.clone(), final_value.clone());
        context.insert(binding.name.clone(), final_value);
    }
    Ok(locals)
}

fn render_let_binding_for_eval(
    binding: &LetBinding,
    render: &RecipeRenderContext<'_>,
    _registry: &RecipeRegistry,
    context: &BTreeMap<String, String>,
) -> Result<LetBinding, String> {
    Ok(LetBinding {
        name: binding.name.clone(),
        query: binding
            .query
            .as_ref()
            .map(|q| render_query_op_public(q, render, context))
            .transpose()?,
        capture: binding
            .capture
            .as_ref()
            .map(|c| render_template(c, context, render.maps, render.vars))
            .transpose()?,
        extract: binding.extract,
        on_no_match: binding.on_no_match,
        on_many_matches: binding.on_many_matches,
        join: binding.join.clone(),
        r#as: binding.r#as.clone(),
    })
}

fn render_single_edit_op(
    op: &EditOp,
    render: &RecipeRenderContext<'_>,
    locals: &BTreeMap<String, String>,
) -> Result<EditOp, String> {
    let ctx = merge_args_and_locals(render.args, locals);
    let mut op = op.clone();
    match &mut op {
        EditOp::Insert(insert) => {
            insert.query = render_query_op_public(&insert.query, render, &ctx)?;
            insert.capture = render_template(&insert.capture, &ctx, render.maps, render.vars)?;
            insert.text = render_template(&insert.text, &ctx, render.maps, render.vars)?;
        }
        EditOp::Replace(replace) => {
            replace.query = render_query_op_public(&replace.query, render, &ctx)?;
            replace.capture = render_template(&replace.capture, &ctx, render.maps, render.vars)?;
            replace.text = render_template(&replace.text, &ctx, render.maps, render.vars)?;
        }
        EditOp::Remove(remove) => {
            remove.query = render_query_op_public(&remove.query, render, &ctx)?;
            remove.capture = render_template(&remove.capture, &ctx, render.maps, render.vars)?;
        }
        EditOp::Unknown(_, _) => {}
    }
    Ok(op)
}

fn merge_args_and_locals(
    recipe_args: &BTreeMap<String, String>,
    locals: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut merged = recipe_args.clone();
    for (k, v) in locals {
        merged.insert(k.clone(), v.clone());
    }
    merged
}

fn engine_error_to_string(error: EngineError) -> String {
    error.to_string()
}
