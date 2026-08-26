use std::collections::BTreeMap;

use codemod_recipe_core::apply::file_change::{IfExists, IfMissing};
use codemod_recipe_engine::engine::{Engine, QueryContext};
use codemod_recipe_engine::ops::{
    EditOp, InsertAnchor, InsertOp, QuerySpec as EngineQuerySpec, RemoveOp, ReplaceOp,
};
use codemod_recipe_lua::host_api::specs::{
    Anchor, ApplyTemplateSpec, CreateSpec, DeleteSpec, IfExists as LuaIfExists,
    IfMissing as LuaIfMissing, InsertSpec, PostExecSpec, QuerySpec as RawQuerySpec, QueryTableSpec,
    ReadSpec, RemoveSpec, RenderFileTemplateSpec, ReplaceSpec,
};
use codemod_recipe_lua::model::TranscriptEvent;
use codemod_recipe_lua_macros::host_method;
use mlua::{Lua, Value};

use crate::engine::post_execution::run_post_execution_command;
use crate::host_api::context::{bump_ops, SharedHost};
use crate::template::{render_template, render_template_file};

#[host_method(
    name = "insert",
    description = "Insert text at a tree-sitter capture (`path`, `query`, `capture`, `text`).",
    spec = InsertSpec,
    return_type = Nil,
)]
pub(crate) fn do_insert(host: &SharedHost, _lua: &Lua, spec: InsertSpec) -> Result<Value, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    bump_ops(&mut inner)?;
    let query = query_spec_to_engine(&spec.query, &inner.queries)?;
    let anchor = match spec.anchor.unwrap_or_default() {
        Anchor::Start => InsertAnchor::Start,
        Anchor::End => InsertAnchor::End,
    };
    let op = EditOp::Insert(InsertOp {
        query,
        capture: spec.capture,
        anchor,
        text: spec.text,
    });
    let path = spec.path;
    let next = apply_source_op(&mut inner, &path, spec.language.as_deref(), &op)?;
    inner.apply_edit(&path, |_| Ok(next))?;
    inner.transcript.push(TranscriptEvent {
        kind: "insert".to_string(),
        detail: path,
        included_in_preview: true,
    });
    Ok(Value::Nil)
}

#[host_method(
    name = "replace",
    description = "Replace a capture span with `text`.",
    spec = ReplaceSpec,
    return_type = Nil,
)]
pub(crate) fn do_replace(
    host: &SharedHost,
    _lua: &Lua,
    spec: ReplaceSpec,
) -> Result<Value, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    bump_ops(&mut inner)?;
    let query = query_spec_to_engine(&spec.query, &inner.queries)?;
    let op = EditOp::Replace(ReplaceOp {
        query,
        capture: spec.capture,
        text: spec.text,
        include_leading_trivia: spec.include_leading_trivia.unwrap_or(false),
    });
    let path = spec.path;
    let next = apply_source_op(&mut inner, &path, spec.language.as_deref(), &op)?;
    inner.apply_edit(&path, |_| Ok(next))?;
    inner.transcript.push(TranscriptEvent {
        kind: "replace".to_string(),
        detail: path,
        included_in_preview: true,
    });
    Ok(Value::Nil)
}

#[host_method(
    name = "remove",
    description = "Remove a capture span.",
    spec = RemoveSpec,
    return_type = Nil,
)]
pub(crate) fn do_remove(host: &SharedHost, _lua: &Lua, spec: RemoveSpec) -> Result<Value, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    bump_ops(&mut inner)?;
    let query = query_spec_to_engine(&spec.query, &inner.queries)?;
    let op = EditOp::Remove(RemoveOp {
        query,
        capture: spec.capture,
        include_leading_trivia: spec.include_leading_trivia.unwrap_or(false),
    });
    let path = spec.path;
    let next = apply_source_op(&mut inner, &path, spec.language.as_deref(), &op)?;
    inner.apply_edit(&path, |_| Ok(next))?;
    inner.transcript.push(TranscriptEvent {
        kind: "remove".to_string(),
        detail: path,
        included_in_preview: true,
    });
    Ok(Value::Nil)
}

#[host_method(
    name = "create",
    description = "Create a file (`path` + `content` or `template`).",
    spec = CreateSpec,
    return_type = Nil,
)]
pub(crate) fn do_create(host: &SharedHost, _lua: &Lua, spec: CreateSpec) -> Result<Value, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    bump_ops(&mut inner)?;
    let content = spec.content.or(spec.template).unwrap_or_default();
    let if_exists = match spec.if_exists.unwrap_or_default() {
        LuaIfExists::Fail => IfExists::Fail,
        LuaIfExists::Skip => IfExists::Skip,
    };
    let path = spec.path;
    inner.create(&path, content, if_exists)?;
    inner.transcript.push(TranscriptEvent {
        kind: "create".to_string(),
        detail: path,
        included_in_preview: true,
    });
    Ok(Value::Nil)
}

#[host_method(
    name = "delete",
    description = "Delete a file.",
    spec = DeleteSpec,
    return_type = Nil,
)]
pub(crate) fn do_delete(host: &SharedHost, _lua: &Lua, spec: DeleteSpec) -> Result<Value, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    bump_ops(&mut inner)?;
    let if_missing = match spec.if_missing.unwrap_or_default() {
        LuaIfMissing::Fail => IfMissing::Fail,
        LuaIfMissing::Skip => IfMissing::Skip,
    };
    let path = spec.path;
    inner.delete(&path, if_missing)?;
    inner.transcript.push(TranscriptEvent {
        kind: "delete".to_string(),
        detail: path,
        included_in_preview: true,
    });
    Ok(Value::Nil)
}

#[host_method(
    name = "read",
    description = "Read file contents (string path or `{ path }`).",
    spec = ReadSpec,
    return_type = String,
    accept_string = true,
)]
pub(crate) fn do_read(host: &SharedHost, lua: &Lua, spec: ReadSpec) -> Result<Value, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    let text = inner.current_content(&spec.path)?;
    inner.transcript.push(TranscriptEvent {
        kind: "read".to_string(),
        detail: spec.path,
        included_in_preview: true,
    });
    lua.create_string(&text)
        .map(Value::String)
        .map_err(|e| e.to_string())
}

#[host_method(
    name = "query",
    description = "Run a tree-sitter query; returns `{ start, [\"end\"], text }` spans.",
    spec = QueryTableSpec,
    return_type = QuerySpans,
)]
pub(crate) fn do_query(
    host: &SharedHost,
    lua: &Lua,
    spec: QueryTableSpec,
) -> Result<Value, String> {
    let capture = spec
        .capture
        .as_deref()
        .ok_or_else(|| "query requires capture".to_string())?;
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    let query = query_spec_to_engine(&spec.query, &inner.queries)?;
    let lang_id = inner
        .language_registry
        .resolve_language_id(spec.language.as_deref(), &spec.path)
        .map_err(|e| e.to_string())?;
    let recipe_path = inner.recipe_path.clone();
    let codemod = inner.codemod_root.clone();
    let source = inner.current_content(&spec.path)?;
    let qctx = QueryContext {
        recipe_file: recipe_path.as_deref(),
        codemod_root: &codemod,
    };
    let engine = inner
        .language_registry
        .get(&lang_id)
        .map_err(|e| e.to_string())?;
    let spans = engine
        .query_capture_spans(&qctx, &source, &query, capture)
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
        detail: format!("{}#{} n={}", spec.path, capture, spans.len()),
        included_in_preview: true,
    });
    Ok(Value::Table(out))
}

#[host_method(
    name = "queryExists",
    description = "True if the query matches at least once.",
    spec = QueryTableSpec,
    return_type = Boolean,
)]
pub(crate) fn do_query_exists(
    host: &SharedHost,
    _lua: &Lua,
    spec: QueryTableSpec,
) -> Result<Value, String> {
    Ok(Value::Boolean(query_count(host, &spec)? > 0))
}

#[host_method(
    name = "queryCount",
    description = "Number of matches (optional `capture`).",
    spec = QueryTableSpec,
    return_type = Integer,
)]
pub(crate) fn do_query_count(
    host: &SharedHost,
    _lua: &Lua,
    spec: QueryTableSpec,
) -> Result<Value, String> {
    Ok(Value::Integer(query_count(host, &spec)? as i64))
}

#[host_method(
    name = "expectMatch",
    description = "Error if the query has zero matches.",
    spec = QueryTableSpec,
    return_type = Nil,
)]
pub(crate) fn do_expect_match(
    host: &SharedHost,
    _lua: &Lua,
    spec: QueryTableSpec,
) -> Result<Value, String> {
    expect_match(host, &spec, true)
}

#[host_method(
    name = "expectNoMatch",
    description = "Error if the query has any match.",
    spec = QueryTableSpec,
    return_type = Nil,
)]
pub(crate) fn do_expect_no_match(
    host: &SharedHost,
    _lua: &Lua,
    spec: QueryTableSpec,
) -> Result<Value, String> {
    expect_match(host, &spec, false)
}

fn expect_match(host: &SharedHost, spec: &QueryTableSpec, want: bool) -> Result<Value, String> {
    let n = query_count(host, spec)?;
    if want && n == 0 {
        return Err("expected query to match".to_string());
    }
    if !want && n > 0 {
        return Err("expected query not to match".to_string());
    }
    Ok(Value::Nil)
}

#[host_method(
    name = "postExec",
    description = "Shell command after apply (Jinja-rendered); preview records only.",
    spec = PostExecSpec,
    return_type = Nil,
)]
pub(crate) fn do_post_exec(
    host: &SharedHost,
    _lua: &Lua,
    spec: PostExecSpec,
) -> Result<Value, String> {
    let cmd = spec
        .command
        .or(spec.path)
        .ok_or_else(|| "postExec requires command or path".to_string())?;
    post_exec_cmd(host, &cmd)?;
    Ok(Value::Nil)
}

#[host_method(
    name = "applyTemplate",
    description = "Render a MiniJinja string template.",
    spec = ApplyTemplateSpec,
    return_type = String,
)]
pub(crate) fn do_apply_template(
    host: &SharedHost,
    lua: &Lua,
    spec: ApplyTemplateSpec,
) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let mut args = inner.args.clone();
    args.extend(spec.env.map(|e| e.into_inner()).unwrap_or_default());
    let rendered = render_template(&spec.template, &args, &inner.maps, &inner.vars)?;
    lua.create_string(&rendered)
        .map(Value::String)
        .map_err(|e| e.to_string())
}

#[host_method(
    name = "renderFileTemplate",
    description = "Render a MiniJinja template file.",
    spec = RenderFileTemplateSpec,
    return_type = String,
)]
pub(crate) fn do_render_file(
    host: &SharedHost,
    lua: &Lua,
    spec: RenderFileTemplateSpec,
) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let mut args = inner.args.clone();
    args.extend(spec.env.map(|e| e.into_inner()).unwrap_or_default());
    let rendered = render_template_file(
        &spec.path,
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

#[host_method(
    name = "map",
    description = "Lookup a registered map by id → string→string table.",
    string_id,
    return_type = Table,
)]
pub(crate) fn do_map(host: &SharedHost, lua: &Lua, id: String) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let map = inner
        .maps
        .get(&id)
        .ok_or_else(|| format!("unknown map '{id}'"))?;
    string_map_to_table(lua, map)
}

#[host_method(
    name = "var",
    description = "Lookup registered variables by id → string→string table.",
    string_id,
    return_type = Table,
)]
pub(crate) fn do_var(host: &SharedHost, lua: &Lua, id: String) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let map = inner
        .vars
        .get(&id)
        .ok_or_else(|| format!("unknown var '{id}'"))?;
    string_map_to_table(lua, map)
}

#[host_method(
    name = "queryLib",
    description = "Lookup a registered query string by id (e.g. `lib.queryName`).",
    string_id,
    return_type = String,
)]
pub(crate) fn do_query_lib(host: &SharedHost, lua: &Lua, id: String) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let q = inner
        .queries
        .get(&id)
        .ok_or_else(|| format!("unknown query '{id}'"))?;
    lua.create_string(q)
        .map(Value::String)
        .map_err(|e| e.to_string())
}

#[host_method(
    name = "assert",
    description = "Runtime assertion (`cond`, optional message).",
    assert,
    return_type = Nil,
)]
pub(crate) fn do_assert(
    _host: &SharedHost,
    _lua: &Lua,
    cond: bool,
    msg: Option<String>,
) -> Result<Value, String> {
    if cond {
        Ok(Value::Nil)
    } else {
        Err(msg.unwrap_or_else(|| "assertion failed".to_string()))
    }
}

pub(crate) fn post_exec_cmd(host: &SharedHost, cmd: &str) -> Result<(), String> {
    use crate::host_api::context::RunKind;

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

fn query_spec_to_engine(
    spec: &RawQuerySpec,
    queries: &BTreeMap<String, String>,
) -> Result<EngineQuerySpec, String> {
    let mut steps: Vec<String> = spec
        .iter()
        .map(|raw| queries.get(raw).cloned().unwrap_or_else(|| raw.to_string()))
        .collect();
    if steps.len() <= 1 {
        Ok(EngineQuerySpec::single(steps.pop().unwrap_or_default()))
    } else {
        Ok(EngineQuerySpec::Chain(steps))
    }
}

fn apply_source_op(
    inner: &mut crate::host_api::context::HostInner,
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

fn query_count(host: &SharedHost, spec: &QueryTableSpec) -> Result<usize, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    let query = query_spec_to_engine(&spec.query, &inner.queries)?;
    let lang_id = inner
        .language_registry
        .resolve_language_id(spec.language.as_deref(), &spec.path)
        .map_err(|e| e.to_string())?;
    let recipe_path = inner.recipe_path.clone();
    let codemod = inner.codemod_root.clone();
    let source = inner.current_content(&spec.path)?;
    let qctx = QueryContext {
        recipe_file: recipe_path.as_deref(),
        codemod_root: &codemod,
    };
    let engine = inner
        .language_registry
        .get(&lang_id)
        .map_err(|e| e.to_string())?;
    engine
        .query_match_count(&qctx, &source, &query, spec.capture.as_deref())
        .map_err(|e| e.to_string())
}

fn string_map_to_table(lua: &Lua, map: &BTreeMap<String, String>) -> Result<Value, String> {
    let t = lua.create_table().map_err(|e| e.to_string())?;
    for (k, v) in map {
        t.set(k.as_str(), v.as_str()).map_err(|e| e.to_string())?;
    }
    Ok(Value::Table(t))
}
