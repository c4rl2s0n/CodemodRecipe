//! Luau recipe execution: shared `ctx` methods with [`RunKind`] preview vs apply.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;

use codemod_recipe_core::apply::file_change::{FileChange, IfExists, IfMissing};
use codemod_recipe_engine::engine::{Engine, QueryContext};
use codemod_recipe_engine::ops::{EditOp, InsertAnchor, InsertOp, QuerySpec, RemoveOp, ReplaceOp};
use codemod_recipe_engine::{LanguageRegistry, RegistryConfig};
use codemod_recipe_lua::json_lua::{json_to_lua, lua_to_json};
use codemod_recipe_lua::model::{Recipe, TranscriptEvent};
use mlua::{Lua, Table, Value};

use crate::engine::post_execution::run_post_execution_command;
use crate::engine::working_tree::WorkingTree;
use crate::sandbox::path_sandbox::PathSandbox;
use crate::template::{render_template, render_template_file};
use codemod_recipe_lua::{eval_return_table, new_sandbox};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunKind {
    Preview,
    Apply,
}

pub struct RecipeRunResult {
    pub changes: Vec<FileChange>,
    pub transcript: Vec<TranscriptEvent>,
    pub saw_post_exec: bool,
}

struct HostInner {
    kind: RunKind,
    tree: WorkingTree,
    sandbox: PathSandbox,
    language_registry: LanguageRegistry,
    args: BTreeMap<String, serde_json::Value>,
    maps: BTreeMap<String, BTreeMap<String, String>>,
    vars: BTreeMap<String, BTreeMap<String, String>>,
    queries: BTreeMap<String, String>,
    recipes: BTreeMap<String, Recipe>,
    recipe_path: Option<PathBuf>,
    codemod_root: PathBuf,
    transcript: Vec<TranscriptEvent>,
    saw_post_exec: bool,
    op_count: usize,
}

impl HostInner {
    fn current_content(&mut self, path: &str) -> Result<String, String> {
        self.tree.current_content(&self.sandbox, path)
    }

    fn apply_edit<F>(&mut self, path: &str, transform: F) -> Result<(), String>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        self.tree.apply_edit(&self.sandbox, path, transform)
    }

    fn create(&mut self, path: &str, content: String, if_exists: IfExists) -> Result<(), String> {
        self.tree.create(&self.sandbox, path, content, if_exists)
    }

    fn delete(&mut self, path: &str, if_missing: IfMissing) -> Result<(), String> {
        self.tree.delete(&self.sandbox, path, if_missing)
    }

    fn flush_to_disk(&self) -> Result<Vec<String>, String> {
        self.tree.flush_to_disk(&self.sandbox)
    }

    fn reload_from_disk(&mut self, path: &str) -> Result<(), String> {
        self.tree.reload_from_disk(&self.sandbox, path)
    }
}

const MAX_OPS: usize = 50_000;
type SharedHost = Rc<Mutex<HostInner>>;

pub fn run_recipe(
    kind: RunKind,
    recipe: &Recipe,
    args: &BTreeMap<String, serde_json::Value>,
    maps: BTreeMap<String, BTreeMap<String, String>>,
    vars: BTreeMap<String, BTreeMap<String, String>>,
    queries: BTreeMap<String, String>,
    recipes: BTreeMap<String, Recipe>,
    workspace_root: PathBuf,
    language_config: RegistryConfig,
    codemod_root: PathBuf,
) -> Result<RecipeRunResult, String> {
    let lua = new_sandbox().map_err(|e| e.to_string())?;
    let inner = Rc::new(Mutex::new(HostInner {
        kind,
        tree: WorkingTree::new(),
        sandbox: PathSandbox::new(workspace_root),
        language_registry: LanguageRegistry::with_config(language_config),
        args: args.clone(),
        maps,
        vars,
        queries,
        recipes,
        recipe_path: recipe.path.clone(),
        codemod_root,
        transcript: Vec::new(),
        saw_post_exec: false,
        op_count: 0,
    }));
    let ctx = build_ctx(&lua, Rc::clone(&inner))?;
    invoke_apply(&lua, recipe, ctx)?;
    if !recipe.post_execution.is_empty() {
        for cmd in &recipe.post_execution {
            post_exec_cmd(&inner, cmd)?;
        }
    }
    let mut host = inner.lock().map_err(|e| e.to_string())?;
    if kind == RunKind::Apply && host.saw_post_exec {
        host.flush_to_disk()?;
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

fn invoke_apply(lua: &Lua, recipe: &Recipe, ctx: Table) -> Result<(), String> {
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

fn build_ctx(lua: &Lua, host: SharedHost) -> Result<Table, String> {
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

    set_fn(lua, &ctx, "insert", {
        let host = Rc::clone(&host);
        move |_, (_this, spec): (Value, Table)| do_edit(&host, spec, EditKind::Insert).map_err(rt)
    })?;
    set_fn(lua, &ctx, "replace", {
        let host = Rc::clone(&host);
        move |_, (_this, spec): (Value, Table)| do_edit(&host, spec, EditKind::Replace).map_err(rt)
    })?;
    set_fn(lua, &ctx, "remove", {
        let host = Rc::clone(&host);
        move |_, (_this, spec): (Value, Table)| do_edit(&host, spec, EditKind::Remove).map_err(rt)
    })?;
    set_fn(lua, &ctx, "create", {
        let host = Rc::clone(&host);
        move |_, (_this, spec): (Value, Table)| do_create(&host, spec).map_err(rt)
    })?;
    set_fn(lua, &ctx, "delete", {
        let host = Rc::clone(&host);
        move |_, (_this, spec): (Value, Table)| do_delete(&host, spec).map_err(rt)
    })?;
    set_fn(lua, &ctx, "read", {
        let host = Rc::clone(&host);
        move |lua, (_this, path): (Value, Value)| do_read(&host, lua, path).map_err(rt)
    })?;
    set_fn(lua, &ctx, "query", {
        let host = Rc::clone(&host);
        move |lua, (_this, spec): (Value, Table)| do_query(&host, lua, spec).map_err(rt)
    })?;
    set_fn(lua, &ctx, "queryExists", {
        let host = Rc::clone(&host);
        move |_, (_this, spec): (Value, Table)| do_query_exists(&host, spec).map_err(rt)
    })?;
    set_fn(lua, &ctx, "queryCount", {
        let host = Rc::clone(&host);
        move |_, (_this, spec): (Value, Table)| do_query_count(&host, spec).map_err(rt)
    })?;
    set_fn(lua, &ctx, "postExec", {
        let host = Rc::clone(&host);
        move |_, (_this, spec): (Value, Table)| do_post_exec(&host, spec).map_err(rt)
    })?;
    set_fn(lua, &ctx, "applyTemplate", {
        let host = Rc::clone(&host);
        move |lua, (_this, spec): (Value, Table)| do_apply_template(&host, lua, spec).map_err(rt)
    })?;
    set_fn(lua, &ctx, "renderFileTemplate", {
        let host = Rc::clone(&host);
        move |lua, (_this, spec): (Value, Table)| do_render_file(&host, lua, spec).map_err(rt)
    })?;
    set_fn(lua, &ctx, "map", {
        let host = Rc::clone(&host);
        move |lua, (_this, id): (Value, String)| do_map(&host, lua, &id).map_err(rt)
    })?;
    set_fn(lua, &ctx, "var", {
        let host = Rc::clone(&host);
        move |lua, (_this, id): (Value, String)| do_var(&host, lua, &id).map_err(rt)
    })?;
    set_fn(lua, &ctx, "queryLib", {
        let host = Rc::clone(&host);
        move |lua, (_this, id): (Value, String)| do_query_lib(&host, lua, &id).map_err(rt)
    })?;
    set_fn(lua, &ctx, "assert", {
        move |_, (_this, cond, msg): (Value, bool, Option<String>)| {
            if cond {
                Ok(Value::Nil)
            } else {
                Err(rt(msg.unwrap_or_else(|| "assertion failed".to_string())))
            }
        }
    })?;
    set_fn(lua, &ctx, "expectMatch", {
        let host = Rc::clone(&host);
        move |_, (_this, spec): (Value, Table)| do_expect_match(&host, spec, true).map_err(rt)
    })?;
    set_fn(lua, &ctx, "expectNoMatch", {
        let host = Rc::clone(&host);
        move |_, (_this, spec): (Value, Table)| do_expect_match(&host, spec, false).map_err(rt)
    })?;
    set_fn(lua, &ctx, "run", {
        let host = Rc::clone(&host);
        move |lua, (_this, id, args): (Value, String, Option<Table>)| {
            do_run(&host, lua, &id, args).map_err(rt)
        }
    })?;
    Ok(ctx)
}

fn rt(msg: String) -> mlua::Error {
    mlua::Error::RuntimeError(msg)
}

fn set_fn<F, A, R>(lua: &Lua, ctx: &Table, name: &str, f: F) -> Result<(), String>
where
    F: Fn(&Lua, A) -> Result<R, mlua::Error> + 'static,
    A: mlua::FromLuaMulti,
    R: mlua::IntoLuaMulti,
{
    let fun = lua.create_function(f).map_err(|e| e.to_string())?;
    ctx.set(name, fun).map_err(|e| e.to_string())
}

enum EditKind {
    Insert,
    Replace,
    Remove,
}

fn bump_ops(host: &mut HostInner) -> Result<(), String> {
    host.op_count += 1;
    if host.op_count > MAX_OPS {
        Err("recipe exceeded operation budget".to_string())
    } else {
        Ok(())
    }
}

fn lua_string(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.to_str().map_err(|e| e.to_string())?.to_string()),
        Value::Integer(i) => Ok(i.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Boolean(b) => Ok(b.to_string()),
        other => Err(format!("expected string, got {}", other.type_name())),
    }
}

fn query_spec_from_lua(
    spec: &Table,
    queries: &BTreeMap<String, String>,
) -> Result<QuerySpec, String> {
    match spec.get::<Value>("query") {
        Ok(Value::String(s)) => {
            let raw = s.to_str().map_err(|e| e.to_string())?.to_string();
            if let Some(found) = queries.get(&raw) {
                Ok(QuerySpec::single(found.clone()))
            } else {
                Ok(QuerySpec::single(raw))
            }
        }
        Ok(Value::Table(t)) => {
            let mut steps = Vec::new();
            for v in t.sequence_values::<Value>() {
                let value = v.map_err(|e| e.to_string())?;
                let raw = lua_string(&value)?;
                steps.push(queries.get(&raw).cloned().unwrap_or(raw));
            }
            if steps.len() <= 1 {
                Ok(QuerySpec::single(steps.pop().unwrap_or_default()))
            } else {
                Ok(QuerySpec::Chain(steps))
            }
        }
        _ => Err("query is required".to_string()),
    }
}

fn apply_source_op(
    inner: &mut HostInner,
    path: &str,
    language: Option<&str>,
    op: &EditOp,
) -> Result<String, String> {
    let lang_id = inner
        .language_registry
        .resolve_language_id(language, path)
        .map_err(|e| e.to_string())?;
    let recipe_path = inner.recipe_path.clone();
    let codemod = inner.codemod_root.clone();
    let source = inner.current_content(path)?;
    let qctx = QueryContext {
        recipe_file: recipe_path.as_deref(),
        codemod_root: &codemod,
    };
    let engine: &mut Engine = inner
        .language_registry
        .get(&lang_id)
        .map_err(|e| e.to_string())?;
    engine
        .apply_edit_ops_sequential(&qctx, std::slice::from_ref(op), &source)
        .map_err(|e| e.to_string())
}

fn do_edit(host: &SharedHost, spec: Table, kind: EditKind) -> Result<Value, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    bump_ops(&mut inner)?;
    let path: String = spec
        .get("path")
        .map_err(|_| "edit requires path".to_string())?;
    let language: Option<String> = spec.get("language").ok();
    let query = query_spec_from_lua(&spec, &inner.queries)?;
    let capture: String = spec
        .get("capture")
        .map_err(|_| "edit requires capture".to_string())?;
    let op = match kind {
        EditKind::Insert => {
            let text: String = spec
                .get("text")
                .map_err(|_| "insert requires text".to_string())?;
            let anchor = match spec
                .get::<String>("anchor")
                .unwrap_or_else(|_| "end".to_string())
                .as_str()
            {
                "start" => InsertAnchor::Start,
                _ => InsertAnchor::End,
            };
            EditOp::Insert(InsertOp {
                query,
                capture,
                anchor,
                text,
            })
        }
        EditKind::Replace => {
            let text: String = spec
                .get("text")
                .map_err(|_| "replace requires text".to_string())?;
            EditOp::Replace(ReplaceOp {
                query,
                capture,
                text,
                include_leading_trivia: spec.get("includeLeadingTrivia").unwrap_or(false),
            })
        }
        EditKind::Remove => EditOp::Remove(RemoveOp {
            query,
            capture,
            include_leading_trivia: spec.get("includeLeadingTrivia").unwrap_or(false),
        }),
    };
    let next = apply_source_op(&mut inner, &path, language.as_deref(), &op)?;
    inner.apply_edit(&path, |_| Ok(next))?;
    let kind_name = match op {
        EditOp::Insert(_) => "insert",
        EditOp::Replace(_) => "replace",
        EditOp::Remove(_) => "remove",
    };
    inner.transcript.push(TranscriptEvent {
        kind: kind_name.to_string(),
        detail: path,
        included_in_preview: true,
    });
    Ok(Value::Nil)
}

fn do_create(host: &SharedHost, spec: Table) -> Result<Value, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    bump_ops(&mut inner)?;
    let path: String = spec
        .get("path")
        .map_err(|_| "create requires path".to_string())?;
    let content: String = spec
        .get("content")
        .or_else(|_| spec.get("template"))
        .unwrap_or_default();
    let if_exists = match spec
        .get::<String>("ifExists")
        .unwrap_or_else(|_| "fail".to_string())
        .as_str()
    {
        "skip" => IfExists::Skip,
        _ => IfExists::Fail,
    };
    inner.create(&path, content, if_exists)?;
    inner.transcript.push(TranscriptEvent {
        kind: "create".to_string(),
        detail: path,
        included_in_preview: true,
    });
    Ok(Value::Nil)
}

fn do_delete(host: &SharedHost, spec: Table) -> Result<Value, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    bump_ops(&mut inner)?;
    let path: String = spec
        .get("path")
        .map_err(|_| "delete requires path".to_string())?;
    let if_missing = match spec
        .get::<String>("ifMissing")
        .unwrap_or_else(|_| "fail".to_string())
        .as_str()
    {
        "skip" => IfMissing::Skip,
        _ => IfMissing::Fail,
    };
    inner.delete(&path, if_missing)?;
    inner.transcript.push(TranscriptEvent {
        kind: "delete".to_string(),
        detail: path,
        included_in_preview: true,
    });
    Ok(Value::Nil)
}

fn path_from_value(value: Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.to_str().map_err(|e| e.to_string())?.to_string()),
        Value::Table(t) => t.get("path").map_err(|_| "read requires path".to_string()),
        other => Err(format!("read requires path, got {}", other.type_name())),
    }
}

fn do_read(host: &SharedHost, lua: &Lua, path: Value) -> Result<Value, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    let path = path_from_value(path)?;
    let text = inner.current_content(&path)?;
    inner.transcript.push(TranscriptEvent {
        kind: "read".to_string(),
        detail: path,
        included_in_preview: true,
    });
    lua.create_string(&text)
        .map(Value::String)
        .map_err(|e| e.to_string())
}

fn do_query(host: &SharedHost, lua: &Lua, spec: Table) -> Result<Value, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    let capture: String = spec
        .get("capture")
        .map_err(|_| "query requires capture".to_string())?;
    let path: String = spec
        .get("path")
        .map_err(|_| "query requires path".to_string())?;
    let language: Option<String> = spec.get("language").ok();
    let query = query_spec_from_lua(&spec, &inner.queries)?;
    let lang_id = inner
        .language_registry
        .resolve_language_id(language.as_deref(), &path)
        .map_err(|e| e.to_string())?;
    let recipe_path = inner.recipe_path.clone();
    let codemod = inner.codemod_root.clone();
    let source = inner.current_content(&path)?;
    let qctx = QueryContext {
        recipe_file: recipe_path.as_deref(),
        codemod_root: &codemod,
    };
    let engine = inner
        .language_registry
        .get(&lang_id)
        .map_err(|e| e.to_string())?;
    let spans = engine
        .query_capture_spans(&qctx, &source, &query, &capture)
        .map_err(|e| e.to_string())?;
    let out = lua.create_table().map_err(|e| e.to_string())?;
    for (i, span) in spans.iter().enumerate() {
        let row = lua.create_table().map_err(|e| e.to_string())?;
        row.set("start", span.start as i64)
            .map_err(|e| e.to_string())?;
        row.set("end", span.end as i64).map_err(|e| e.to_string())?;
        row.set("text", &source[span.start..span.end])
            .map_err(|e| e.to_string())?;
        out.set(i + 1, row).map_err(|e| e.to_string())?;
    }
    inner.transcript.push(TranscriptEvent {
        kind: "query".to_string(),
        detail: format!("{path}#{capture} n={}", spans.len()),
        included_in_preview: true,
    });
    Ok(Value::Table(out))
}

fn do_query_exists(host: &SharedHost, spec: Table) -> Result<Value, String> {
    Ok(Value::Boolean(query_count(host, spec)? > 0))
}

fn do_query_count(host: &SharedHost, spec: Table) -> Result<Value, String> {
    Ok(Value::Integer(query_count(host, spec)? as i64))
}

fn query_count(host: &SharedHost, spec: Table) -> Result<usize, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    let path: String = spec
        .get("path")
        .map_err(|_| "query requires path".to_string())?;
    let language: Option<String> = spec.get("language").ok();
    let query = query_spec_from_lua(&spec, &inner.queries)?;
    let capture: Option<String> = spec.get("capture").ok();
    let lang_id = inner
        .language_registry
        .resolve_language_id(language.as_deref(), &path)
        .map_err(|e| e.to_string())?;
    let recipe_path = inner.recipe_path.clone();
    let codemod = inner.codemod_root.clone();
    let source = inner.current_content(&path)?;
    let qctx = QueryContext {
        recipe_file: recipe_path.as_deref(),
        codemod_root: &codemod,
    };
    let engine = inner
        .language_registry
        .get(&lang_id)
        .map_err(|e| e.to_string())?;
    engine
        .query_match_count(&qctx, &source, &query, capture.as_deref())
        .map_err(|e| e.to_string())
}

fn do_expect_match(host: &SharedHost, spec: Table, want: bool) -> Result<Value, String> {
    let n = query_count(host, spec)?;
    if want && n == 0 {
        return Err("expected query to match".to_string());
    }
    if !want && n > 0 {
        return Err("expected query not to match".to_string());
    }
    Ok(Value::Nil)
}

fn do_post_exec(host: &SharedHost, spec: Table) -> Result<Value, String> {
    let cmd: String = spec
        .get("command")
        .or_else(|_| spec.get("path"))
        .map_err(|_| "postExec requires command or path".to_string())?;
    post_exec_cmd(host, &cmd)?;
    Ok(Value::Nil)
}

fn post_exec_cmd(host: &SharedHost, cmd: &str) -> Result<(), String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    bump_ops(&mut inner)?;
    inner.saw_post_exec = true;
    match inner.kind {
        RunKind::Preview => {
            inner.transcript.push(TranscriptEvent {
                kind: "postExec".to_string(),
                detail: cmd.to_string(),
                included_in_preview: false,
            });
            Ok(())
        }
        RunKind::Apply => {
            inner.flush_to_disk()?;
            let workspace = inner.sandbox.workspace_root().to_path_buf();
            let codemod = inner.codemod_root.clone();
            let recipe_file = inner.recipe_path.clone();
            let args = inner.args.clone();
            let maps = inner.maps.clone();
            let vars = inner.vars.clone();
            run_post_execution_command(
                cmd,
                &args,
                &maps,
                &vars,
                &workspace,
                &codemod,
                recipe_file.as_deref(),
            )?;
            let paths = inner.tree.staged_paths();
            for p in paths {
                inner.reload_from_disk(&p)?;
            }
            inner.transcript.push(TranscriptEvent {
                kind: "postExec".to_string(),
                detail: cmd.to_string(),
                included_in_preview: false,
            });
            Ok(())
        }
    }
}

fn extra_env(spec: &Table) -> Result<BTreeMap<String, serde_json::Value>, String> {
    match spec.get::<Value>("env") {
        Ok(Value::Table(t)) => {
            let mut out = BTreeMap::new();
            for pair in t.pairs::<String, Value>() {
                let (k, v) = pair.map_err(|e| e.to_string())?;
                out.insert(k, lua_to_json(&v)?);
            }
            Ok(out)
        }
        _ => Ok(BTreeMap::new()),
    }
}

fn do_apply_template(host: &SharedHost, lua: &Lua, spec: Table) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let template: String = spec
        .get("template")
        .map_err(|_| "applyTemplate requires template".to_string())?;
    let mut args = inner.args.clone();
    args.extend(extra_env(&spec)?);
    let rendered = render_template(&template, &args, &inner.maps, &inner.vars)?;
    lua.create_string(&rendered)
        .map(Value::String)
        .map_err(|e| e.to_string())
}

fn do_render_file(host: &SharedHost, lua: &Lua, spec: Table) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let path: String = spec
        .get("path")
        .map_err(|_| "renderFileTemplate requires path".to_string())?;
    let mut args = inner.args.clone();
    args.extend(extra_env(&spec)?);
    let rendered = render_template_file(
        &path,
        &args,
        &inner.maps,
        &inner.vars,
        &inner.codemod_root,
        inner.recipe_path.as_deref(),
    )?;
    lua.create_string(&rendered)
        .map(Value::String)
        .map_err(|e| e.to_string())
}

fn string_map_to_table(lua: &Lua, map: &BTreeMap<String, String>) -> Result<Value, String> {
    let t = lua.create_table().map_err(|e| e.to_string())?;
    for (k, v) in map {
        t.set(k.as_str(), v.as_str()).map_err(|e| e.to_string())?;
    }
    Ok(Value::Table(t))
}

fn do_map(host: &SharedHost, lua: &Lua, id: &str) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let map = inner
        .maps
        .get(id)
        .ok_or_else(|| format!("unknown map '{id}'"))?;
    string_map_to_table(lua, map)
}

fn do_var(host: &SharedHost, lua: &Lua, id: &str) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let map = inner
        .vars
        .get(id)
        .ok_or_else(|| format!("unknown var '{id}'"))?;
    string_map_to_table(lua, map)
}

fn do_query_lib(host: &SharedHost, lua: &Lua, id: &str) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let q = inner
        .queries
        .get(id)
        .ok_or_else(|| format!("unknown query '{id}'"))?;
    lua.create_string(q)
        .map(Value::String)
        .map_err(|e| e.to_string())
}

fn do_run(host: &SharedHost, lua: &Lua, id: &str, args: Option<Table>) -> Result<Value, String> {
    let child = {
        let inner = host.lock().map_err(|e| e.to_string())?;
        inner
            .recipes
            .get(id)
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
    if let Some(tbl) = args {
        for pair in tbl.pairs::<String, Value>() {
            let (k, v) = pair.map_err(|e| e.to_string())?;
            child_args.insert(k, lua_to_json(&v)?);
        }
    }
    {
        let mut inner = host.lock().map_err(|e| e.to_string())?;
        inner.args.clone_from(&child_args);
        inner.transcript.push(TranscriptEvent {
            kind: "run".to_string(),
            detail: id.to_string(),
            included_in_preview: true,
        });
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
