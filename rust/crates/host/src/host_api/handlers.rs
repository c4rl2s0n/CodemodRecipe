use std::collections::BTreeMap;

use codemod_recipe_core::apply::file_change::{IfExists, IfMissing};
use codemod_recipe_engine::engine::{CaptureSpan, Engine, QueryContext};
use codemod_recipe_engine::ops::{
    EditOp, InsertAnchor, InsertOp, QuerySpec as EngineQuerySpec, RemoveOp, ReplaceOp,
};
use codemod_recipe_lua::host_api::specs::{
    Anchor, CreateSpec, DeleteSpec, IfExists as LuaIfExists, IfMissing as LuaIfMissing, InsertSpec,
    MatchFileRow, PostExecSpec, QueryCaptureSpec, QueryHit, QueryMatchRow,
    QuerySpec as RawQuerySpec, QueryTableSpec, ReadSpec, RemoveSpec, ReplaceSpec, TemplateSpec,
};
use codemod_recipe_lua::model::{TranscriptEvent, TranscriptStats};
use codemod_recipe_lua_macros::host_method;
use mlua::{Lua, Table, Value};
use serde_json::json;

use crate::engine::post_execution::run_post_execution_command;
use crate::host_api::context::{bump_ops, SharedHost};
use crate::template::{render_template, render_template_file};

const TRANSCRIPT_TEXT_LIMIT: usize = 4096;

fn count_lines(s: &str) -> i64 {
    if s.is_empty() {
        0
    } else {
        s.lines().count() as i64
    }
}

fn trunc_text(s: &str) -> String {
    if s.len() <= TRANSCRIPT_TEXT_LIMIT {
        s.to_string()
    } else {
        let mut end = TRANSCRIPT_TEXT_LIMIT;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &s[..end])
    }
}

fn query_label(query: &RawQuerySpec) -> Option<String> {
    let steps: Vec<&str> = query.iter().collect();
    if steps.is_empty() {
        None
    } else if steps.len() == 1 {
        Some(trunc_text(steps[0]))
    } else {
        Some(format!("{} steps", steps.len()))
    }
}

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
    let query_detail = query_label(&spec.query);
    let anchor = match spec.anchor.unwrap_or_default() {
        Anchor::Start => InsertAnchor::Start,
        Anchor::End => InsertAnchor::End,
    };
    let anchor_label = match anchor {
        InsertAnchor::Start => "start",
        InsertAnchor::End => "end",
    };
    let capture = spec.capture.clone();
    let text = spec.text.clone();
    let op = EditOp::Insert(InsertOp {
        query,
        capture: spec.capture,
        anchor,
        text: spec.text,
    });
    let path = spec.path;
    let next = apply_source_op(&mut inner, &path, spec.language.as_deref(), &op)?;
    inner.apply_edit(&path, |_| Ok(next))?;
    let mut payload = json!({
        "capture": capture,
        "anchor": anchor_label,
        "text": trunc_text(&text),
    });
    if let Some(q) = query_detail {
        payload["query"] = json!(q);
    }
    inner.transcript.push(TranscriptEvent::edit(
        "insert",
        path,
        TranscriptStats {
            added_lines: Some(count_lines(&text)),
            removed_lines: None,
        },
        payload,
    ));
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
    let query_detail = query_label(&spec.query);
    let capture = spec.capture.clone();
    let text = spec.text.clone();
    let removed = capture_texts(
        &mut inner,
        &spec.path,
        spec.language.as_deref(),
        &query,
        &capture,
    )?;
    let removed_joined = removed.join("\n---\n");
    let match_count = removed.len() as i64;
    let op = EditOp::Replace(ReplaceOp {
        query,
        capture: spec.capture,
        text: spec.text,
        include_leading_trivia: spec.include_leading_trivia.unwrap_or(false),
    });
    let path = spec.path;
    let next = apply_source_op(&mut inner, &path, spec.language.as_deref(), &op)?;
    inner.apply_edit(&path, |_| Ok(next))?;
    let mut payload = json!({
        "capture": capture,
        "text": trunc_text(&text),
        "removedText": trunc_text(&removed_joined),
    });
    if let Some(q) = query_detail {
        payload["query"] = json!(q);
    }
    inner.transcript.push(TranscriptEvent::edit(
        "replace",
        path,
        TranscriptStats {
            added_lines: Some(count_lines(&text) * match_count),
            removed_lines: Some(removed.iter().map(|s| count_lines(s)).sum()),
        },
        payload,
    ));
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
    let query_detail = query_label(&spec.query);
    let capture = spec.capture.clone();
    let removed = capture_texts(
        &mut inner,
        &spec.path,
        spec.language.as_deref(),
        &query,
        &capture,
    )?;
    let removed_joined = removed.join("\n---\n");
    let op = EditOp::Remove(RemoveOp {
        query,
        capture: spec.capture,
        include_leading_trivia: spec.include_leading_trivia.unwrap_or(false),
    });
    let path = spec.path;
    let next = apply_source_op(&mut inner, &path, spec.language.as_deref(), &op)?;
    inner.apply_edit(&path, |_| Ok(next))?;
    let mut payload = json!({
        "capture": capture,
        "removedText": trunc_text(&removed_joined),
    });
    if let Some(q) = query_detail {
        payload["query"] = json!(q);
    }
    inner.transcript.push(TranscriptEvent::edit(
        "remove",
        path,
        TranscriptStats {
            added_lines: None,
            removed_lines: Some(removed.iter().map(|s| count_lines(s)).sum()),
        },
        payload,
    ));
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
    let payload = json!({
        "content": trunc_text(&content),
    });
    let stats = TranscriptStats {
        added_lines: Some(count_lines(&content)),
        removed_lines: None,
    };
    inner.create(&path, content, if_exists)?;
    inner
        .transcript
        .push(TranscriptEvent::edit("create", path, stats, payload));
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
    inner
        .transcript
        .push(TranscriptEvent::simple("delete", path));
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
    let chars = text.chars().count();
    let bytes = text.len();
    inner.transcript.push(TranscriptEvent::with_payload(
        "read",
        spec.path.clone(),
        json!({
            "chars": chars,
            "bytes": bytes,
        }),
    ));
    lua.create_string(&text)
        .map(Value::String)
        .map_err(|e| e.to_string())
}

#[host_method(
    name = "query",
    description = "Run a tree-sitter query; returns hits `{ span = { from, to }, text, kind }`.",
    spec = QueryCaptureSpec,
    return_type = Array(QueryHit),
)]
pub(crate) fn do_query(
    host: &SharedHost,
    lua: &Lua,
    spec: QueryCaptureSpec,
) -> Result<Value, String> {
    let capture = spec.capture.as_str();
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
        let row = hit_table(lua, &source, span)?;
        out.set(i + 1, row).map_err(|e| e.to_string())?;
    }
    let first_snippet = spans
        .first()
        .map(|span| trunc_text(&source[span.start..span.end]));
    inner.transcript.push(TranscriptEvent::with_payload(
        "query",
        format!("{}#{} n={}", spec.path, capture, spans.len()),
        json!({
            "path": spec.path,
            "capture": capture,
            "matchCount": spans.len(),
            "firstMatch": first_snippet,
        }),
    ));
    Ok(Value::Table(out))
}

#[host_method(
    name = "queryMatches",
    description = "Per-match rows keyed by @capture name; multi → array, single → hit.",
    spec = QueryTableSpec,
    return_type = Array(QueryMatchRow),
)]
pub(crate) fn do_query_matches(
    host: &SharedHost,
    lua: &Lua,
    spec: QueryTableSpec,
) -> Result<Value, String> {
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
    let rows = engine
        .query_match_captures(&qctx, &source, &query)
        .map_err(|e| e.to_string())?;
    let out = lua.create_table().map_err(|e| e.to_string())?;
    for (i, row) in rows.iter().enumerate() {
        let entry = lua.create_table().map_err(|e| e.to_string())?;
        for (name, spans) in row {
            let value = match spans.as_slice() {
                [] => Value::Nil,
                [one] => Value::Table(hit_table(lua, &source, one)?),
                many => {
                    let arr = lua.create_table().map_err(|e| e.to_string())?;
                    for (j, span) in many.iter().enumerate() {
                        arr.set(j + 1, hit_table(lua, &source, span)?)
                            .map_err(|e| e.to_string())?;
                    }
                    Value::Table(arr)
                }
            };
            entry.set(name.as_str(), value).map_err(|e| e.to_string())?;
        }
        out.set(i + 1, entry).map_err(|e| e.to_string())?;
    }
    inner.transcript.push(TranscriptEvent::with_payload(
        "queryMatches",
        format!("{} n={}", spec.path, rows.len()),
        json!({
            "path": spec.path,
            "matchCount": rows.len(),
        }),
    ));
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
    name = "template",
    description = "Render MiniJinja from `inline` or file `path` (exactly one required).",
    spec = TemplateSpec,
    return_type = String,
)]
pub(crate) fn do_template(
    host: &SharedHost,
    lua: &Lua,
    spec: TemplateSpec,
) -> Result<Value, String> {
    let has_path = spec.path.as_ref().is_some_and(|p| !p.is_empty());
    let has_inline = spec.inline.as_ref().is_some_and(|s| !s.is_empty());
    match (has_path, has_inline) {
        (true, false) => {}
        (false, true) => {}
        (true, true) => {
            return Err("template requires exactly one of path or inline".to_string());
        }
        (false, false) => {
            return Err("template requires path or inline".to_string());
        }
    }

    let mut inner = host.lock().map_err(|e| e.to_string())?;
    let mut args = inner.args.clone();
    args.extend(spec.env.map(|e| e.into_inner()).unwrap_or_default());
    let path_label = spec.path.clone();
    let rendered = if has_path {
        render_template_file(
            spec.path.as_deref().unwrap(),
            &args,
            &inner.maps,
            &inner.vars,
            &inner.codemod_root,
            inner.recipe_path.as_deref(),
        )?
    } else {
        render_template(
            spec.inline.as_deref().unwrap(),
            &args,
            &inner.maps,
            &inner.vars,
        )?
    };
    let detail = if has_path {
        format!("path={}", path_label.as_deref().unwrap_or(""))
    } else {
        "inline".to_string()
    };
    inner.transcript.push(TranscriptEvent::with_payload(
        "template",
        detail,
        json!({
            "source": if has_path { "path" } else { "inline" },
            "path": path_label,
            "rendered": trunc_text(&rendered),
        }),
    ));
    lua.create_string(&rendered)
        .map(Value::String)
        .map_err(|e| e.to_string())
}

#[host_method(
    name = "log",
    description = "Append values to the recipe transcript (strings, tables, numbers).",
    variadic,
    recursive = true,
    return_type = Nil,
)]
pub(crate) fn do_log(
    host: &SharedHost,
    _lua: &Lua,
    args: mlua::Variadic<Value>,
) -> Result<Value, String> {
    let (detail, json_parts) = format_log_args_rich(&args)?;
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    bump_ops(&mut inner)?;
    let expandable = !json_parts.is_empty() || detail.len() > 120;
    let payload = if json_parts.len() == 1 {
        Some(json!({ "json": json_parts[0] }))
    } else if json_parts.len() > 1 {
        Some(json!({ "json": json_parts }))
    } else if expandable {
        Some(json!({ "text": trunc_text(&detail) }))
    } else {
        None
    };
    inner.transcript.push(TranscriptEvent {
        kind: "log".to_string(),
        detail,
        included_in_preview: true,
        stats: None,
        payload,
        expandable,
    });
    Ok(Value::Nil)
}

fn format_log_args_rich(
    args: &mlua::Variadic<Value>,
) -> Result<(String, Vec<serde_json::Value>), String> {
    if args.is_empty() {
        return Ok((String::new(), Vec::new()));
    }
    let mut parts = Vec::with_capacity(args.len());
    let mut json_parts = Vec::new();
    for value in args.iter() {
        parts.push(format_log_value(value)?);
        if matches!(value, Value::Table(_)) {
            if let Ok(json) = codemod_recipe_lua::json_lua::lua_to_json(value) {
                json_parts.push(json);
            }
        }
    }
    Ok((parts.join(" "), json_parts))
}

fn format_log_value(value: &Value) -> Result<String, String> {
    use codemod_recipe_lua::json_lua::lua_to_json;

    match value {
        Value::String(s) => Ok(s.to_str().map_err(|e| e.to_string())?.to_string()),
        Value::Nil => Ok("null".to_string()),
        Value::Boolean(b) => Ok(b.to_string()),
        Value::Integer(i) => Ok(i.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Table(_) => match lua_to_json(value) {
            Ok(json) => Ok(json.to_string()),
            Err(_) => Ok("<table>".to_string()),
        },
        Value::Function(_) => Ok("<function>".to_string()),
        Value::Thread(_) => Ok("<thread>".to_string()),
        Value::UserData(_) | Value::LightUserData(_) => Ok("<userdata>".to_string()),
        other => match lua_to_json(other) {
            Ok(json) => Ok(json.to_string()),
            Err(_) => Ok(format!("<{}>", other.type_name())),
        },
    }
}

#[host_method(
    name = "map",
    description = "Lookup a registered map by id → string→string table.",
    string_id,
    return_type = StringMap,
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
    return_type = StringMap,
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
    name = "listFiles",
    description = "List workspace-relative paths matching a `*` glob (one `*` per segment).",
    string_id,
    return_type = StringList,
)]
pub(crate) fn do_list_files(
    host: &SharedHost,
    lua: &Lua,
    pattern: String,
) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let paths =
        crate::sandbox::workspace_glob::glob_workspace(inner.sandbox.workspace_root(), &pattern)?;
    let tbl = lua.create_table().map_err(|e| e.to_string())?;
    for (i, path) in paths.iter().enumerate() {
        tbl.set(i + 1, path.as_str()).map_err(|e| e.to_string())?;
    }
    Ok(Value::Table(tbl))
}

#[host_method(
    name = "matchFiles",
    description = "Match `{placeholder}` path patterns; returns rows with `path` + captures.",
    string_id,
    return_type = Array(MatchFileRow),
)]
pub(crate) fn do_match_files(
    host: &SharedHost,
    lua: &Lua,
    pattern: String,
) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let rows =
        crate::sandbox::workspace_glob::match_workspace(inner.sandbox.workspace_root(), &pattern)?;
    let tbl = lua.create_table().map_err(|e| e.to_string())?;
    for (i, row) in rows.iter().enumerate() {
        let entry = lua.create_table().map_err(|e| e.to_string())?;
        for (k, v) in row {
            entry
                .set(k.as_str(), v.as_str())
                .map_err(|e| e.to_string())?;
        }
        tbl.set(i + 1, entry).map_err(|e| e.to_string())?;
    }
    Ok(Value::Table(tbl))
}

#[host_method(
    name = "filename",
    description = "Basename without extension from a workspace-relative path.",
    string_id,
    return_type = String,
)]
pub(crate) fn do_filename(_host: &SharedHost, lua: &Lua, path: String) -> Result<Value, String> {
    let stem = crate::sandbox::path_template::stem_from_path(&path)
        .ok_or_else(|| format!("cannot derive filename from path: {path}"))?;
    lua.create_string(stem)
        .map(Value::String)
        .map_err(|e| e.to_string())
}

/// Surface metadata for `c:resolve`; runtime binding is in
/// [`crate::lua_runtime::build_ctx`] (two-arg signature).
///
/// See [`crate::host_api::registry::HOST_API_SURFACE`] for why linkme needs
/// `#[allow(unsafe_code)]` under workspace deny.
// SAFETY: linkme element expansion emits `#[unsafe(link_section)]` here.
#[allow(unsafe_code)]
#[::linkme::distributed_slice(crate::host_api::registry::HOST_API_SURFACE)]
static __HOST_API_SURFACE_DO_RESOLVE: crate::host_api::registry::HostMethodSurface =
    crate::host_api::registry::HostMethodSurface {
        name: "resolve",
        description: "Fill `{key}` placeholders in a path template (no MiniJinja / no FS).",
        signature: crate::host_api::registry::HostMethodSignature::Resolve,
        return_type: crate::host_api::registry::HostReturnType::String,
        accept_string: false,
    };

pub(crate) fn do_resolve(
    _host: &SharedHost,
    lua: &Lua,
    template: String,
    vars: Option<Table>,
) -> Result<Value, String> {
    let mut map = BTreeMap::new();
    if let Some(tbl) = vars {
        for pair in tbl.pairs::<String, Value>() {
            let (k, v) = pair.map_err(|e| e.to_string())?;
            let s = match v {
                Value::String(s) => s.to_str().map_err(|e| e.to_string())?.to_string(),
                other => {
                    return Err(format!(
                        "resolve vars must be strings, got {}",
                        other.type_name()
                    ))
                }
            };
            map.insert(k, s);
        }
    }
    let resolved = crate::sandbox::path_template::resolve_template(&template, &map);
    lua.create_string(resolved)
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
                stats: None,
                payload: Some(json!({ "executed": false })),
                expandable: true,
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
                stats: None,
                payload: Some(json!({ "executed": true })),
                expandable: true,
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

fn capture_texts(
    inner: &mut crate::host_api::context::HostInner,
    path: &str,
    language: Option<&str>,
    query: &EngineQuerySpec,
    capture: &str,
) -> Result<Vec<String>, String> {
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
    let engine = inner
        .language_registry
        .get(&lang_id)
        .map_err(|e| e.to_string())?;
    let spans = engine
        .query_capture_spans(&qctx, &source, query, capture)
        .map_err(|e| e.to_string())?;
    Ok(spans
        .into_iter()
        .map(|span| source[span.start..span.end].to_string())
        .collect())
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

fn hit_table(lua: &Lua, source: &str, span: &CaptureSpan) -> Result<Table, String> {
    let row = lua.create_table().map_err(|e| e.to_string())?;
    let span_tbl = lua.create_table().map_err(|e| e.to_string())?;
    span_tbl
        .set("from", span.start as i64)
        .map_err(|e| e.to_string())?;
    span_tbl
        .set("to", span.end as i64)
        .map_err(|e| e.to_string())?;
    row.set("span", span_tbl).map_err(|e| e.to_string())?;
    row.set("text", &source[span.start..span.end])
        .map_err(|e| e.to_string())?;
    row.set("kind", span.kind.as_str())
        .map_err(|e| e.to_string())?;
    Ok(row)
}
