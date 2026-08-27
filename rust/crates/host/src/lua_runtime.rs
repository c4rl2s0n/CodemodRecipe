//! Luau recipe execution: shared `ctx` methods with [`RunKind`] preview vs apply.

use std::rc::Rc;
use std::sync::Mutex;

use codemod_recipe_engine::LanguageRegistry;
use codemod_recipe_lua::json_lua::{json_to_lua, lua_to_json};
use codemod_recipe_lua::model::{Recipe, TranscriptEvent};
use codemod_recipe_lua::{eval_return_table, new_sandbox};
use codemod_recipe_lua_macros::host_method;
use mlua::{Lua, Table, Value};
use serde_json::json;

use crate::engine::working_tree::WorkingTree;
use crate::host_api::context::{HostInner, SharedHost};
pub use crate::host_api::context::{
    RecipeApplyError, RecipeRunRequest, RecipeRunResult, RunKind,
};
use crate::host_api::handlers::{do_log, post_exec_cmd};
use crate::host_api::registry::{rt, set_fn, HOST_METHODS};
use crate::sandbox::path_sandbox::PathSandbox;

pub fn run_recipe(
    recipe: &Recipe,
    request: RecipeRunRequest,
) -> Result<RecipeRunResult, RecipeApplyError> {
    let lua = new_sandbox().map_err(|e| RecipeApplyError {
        message: e.to_string(),
        transcript: Vec::new(),
    })?;
    let kind = request.kind;
    let inner = Rc::new(Mutex::new(HostInner {
        kind,
        tree: WorkingTree::new(),
        sandbox: PathSandbox::new(request.workspace_root),
        language_registry: LanguageRegistry::with_config(request.language_config),
        args: request.args,
        maps: request.maps,
        vars: request.vars,
        queries: request.queries,
        recipes: request.recipes,
        recipe_path: recipe.path.clone(),
        codemod_root: request.codemod_root,
        transcript: Vec::new(),
        saw_post_exec: false,
        op_count: 0,
    }));
    let ctx = build_ctx(&lua, Rc::clone(&inner)).map_err(|message| {
        take_apply_error(&inner, message)
    })?;
    if let Err(message) = invoke_apply(&lua, recipe, ctx) {
        return Err(take_apply_error(&inner, message));
    }
    if !recipe.post_execution.is_empty() {
        for cmd in &recipe.post_execution {
            if let Err(message) = post_exec_cmd(&inner, cmd) {
                return Err(take_apply_error(&inner, message));
            }
        }
    }
    let mut host = inner.lock().map_err(|e| RecipeApplyError {
        message: e.to_string(),
        transcript: Vec::new(),
    })?;
    if kind == RunKind::Apply && host.saw_post_exec {
        if let Err(message) = host.flush_to_disk() {
            let transcript = std::mem::take(&mut host.transcript);
            return Err(RecipeApplyError {
                message,
                transcript,
            });
        }
    }
    let transcript = std::mem::take(&mut host.transcript);
    let saw_post_exec = host.saw_post_exec;
    let tree = std::mem::replace(&mut host.tree, WorkingTree::new());
    Ok(RecipeRunResult {
        changes: tree.finalize(),
        transcript,
        saw_post_exec,
    })
}

fn take_apply_error(host: &SharedHost, message: String) -> RecipeApplyError {
    let transcript = host
        .lock()
        .map(|mut inner| std::mem::take(&mut inner.transcript))
        .unwrap_or_default();
    RecipeApplyError {
        message,
        transcript,
    }
}

pub(crate) fn invoke_apply(lua: &Lua, recipe: &Recipe, ctx: Table) -> Result<(), String> {
    let name = recipe
        .path
        .as_ref()
        .and_then(|p| p.to_str())
        .unwrap_or(recipe.id.as_str());
    let table = eval_return_table(lua, &recipe.source, name).map_err(|e| e.to_string())?;
    let apply: mlua::Function = table
        .get("apply")
        .map_err(|_| "recipe apply function is required".to_string())?;
    apply
        .call::<()>(ctx)
        .map_err(|e| format!("recipe apply failed: {e}"))
}

pub(crate) fn build_ctx(lua: &Lua, host: SharedHost) -> Result<Table, String> {
    let ctx = lua.create_table().map_err(|e| e.to_string())?;
    let args_tbl = lua.create_table().map_err(|e| e.to_string())?;
    {
        let inner = host.lock().map_err(|e| e.to_string())?;
        for (k, v) in &inner.args {
            args_tbl
                .set(k.as_str(), json_to_lua(lua, v)?)
                .map_err(|e| e.to_string())?;
        }
    }
    ctx.set("args", args_tbl).map_err(|e| e.to_string())?;

    for method in HOST_METHODS {
        (method.register)(lua, &ctx, &host)?;
    }

    // `run` is registered manually because it recursively calls `build_ctx`.
    let run_host = Rc::clone(&host);
    set_fn(
        lua,
        &ctx,
        "run",
        move |lua, (_this, id, args): (Value, String, Option<Table>)| {
            do_run(&run_host, lua, id, args).map_err(rt)
        },
    )?;

    // `log` is registered manually for Variadic args.
    let log_host = Rc::clone(&host);
    set_fn(
        lua,
        &ctx,
        "log",
        move |lua, (_this, args): (Value, mlua::Variadic<Value>)| {
            do_log(&log_host, lua, args).map_err(rt)
        },
    )?;

    Ok(ctx)
}

#[host_method(
    name = "run",
    description = "Run a nested recipe by id with optional arg overrides.",
    run,
    recursive = true,
    return_type = Nil,
)]
fn do_run(host: &SharedHost, lua: &Lua, id: String, args: Option<Table>) -> Result<Value, String> {
    let child = {
        let inner = host.lock().map_err(|e| e.to_string())?;
        inner
            .recipes
            .get(&id)
            .cloned()
            .ok_or_else(|| format!("recipe not found: {id}"))?
    };
    let parent_args;
    let mut child_args;
    {
        let inner = host.lock().map_err(|e| e.to_string())?;
        parent_args = inner.args.clone();
        child_args = inner.args.clone();
    }
    let mut override_keys = Vec::new();
    if let Some(tbl) = args {
        for pair in tbl.pairs::<String, Value>() {
            let (k, v) = pair.map_err(|e| e.to_string())?;
            override_keys.push(k.clone());
            child_args.insert(k, lua_to_json(&v)?);
        }
    }
    {
        let mut inner = host.lock().map_err(|e| e.to_string())?;
        inner.args.clone_from(&child_args);
        inner.transcript.push(TranscriptEvent::with_payload(
            "run",
            id.to_string(),
            json!({
                "recipeId": id,
                "overrideArgKeys": override_keys,
            }),
        ));
    }
    invoke_apply(lua, &child, build_ctx(lua, Rc::clone(host))?)?;
    if !child.post_execution.is_empty() {
        for cmd in &child.post_execution {
            post_exec_cmd(host, cmd)?;
        }
    }
    {
        let mut inner = host.lock().map_err(|e| e.to_string())?;
        inner.args = parent_args;
    }
    Ok(Value::Nil)
}
