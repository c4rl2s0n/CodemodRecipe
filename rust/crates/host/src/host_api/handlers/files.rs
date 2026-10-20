use std::collections::BTreeMap;

use codemod_recipe_core::apply::file_change::{IfExists, IfMissing};
use codemod_recipe_lua::host_api::specs::{
    CreateSpec, DeleteSpec, ListFilesSpec, MatchFileRow, ReadSpec, SpanTextSpec,
};
use codemod_recipe_lua::model::{TranscriptEvent, TranscriptStats};
use codemod_recipe_lua_macros::host_method;
use mlua::{Lua, Table, Value};
use serde_json::json;

use crate::host_api::context::{bump_ops, SharedHost};
use super::helpers::{count_lines, string_map_to_table, trunc_text};

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
    let if_exists: IfExists = spec.if_exists.unwrap_or_default().into();
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
    let if_missing: IfMissing = spec.if_missing.unwrap_or_default().into();
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
    name = "spanText",
    description = "Extract text for a 0-based byte span `{ path, span = { from, to } }`. Prefer hit.text from c:query when available.",
    spec = SpanTextSpec,
    return_type = String,
)]
pub(crate) fn do_span_text(
    host: &SharedHost,
    lua: &Lua,
    spec: SpanTextSpec,
) -> Result<Value, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    let text = inner.current_content(&spec.path)?;
    let from = spec.span.from as usize;
    let to = spec.span.to as usize;
    if from > to || to > text.len() {
        return Err(format!(
            "spanText: span [{from}, {to}) out of bounds for {} (len={})",
            spec.path,
            text.len()
        ));
    }
    if !text.is_char_boundary(from) || !text.is_char_boundary(to) {
        return Err(format!(
            "spanText: span [{from}, {to}) is not on a UTF-8 char boundary in {}",
            spec.path
        ));
    }
    let slice = &text[from..to];
    inner.transcript.push(TranscriptEvent::with_payload(
        "spanText",
        format!("{}[{from}..{to}]", spec.path),
        json!({ "chars": slice.chars().count(), "bytes": slice.len() }),
    ));
    lua.create_string(slice)
        .map(Value::String)
        .map_err(|e| e.to_string())
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
    description = "List workspace-relative paths matching globs (`*` / `**`). Pass a string or an array; patterns starting with `!` exclude (union-then-subtract).",
    spec = ListFilesSpec,
    return_type = StringList,
)]
pub(crate) fn do_list_files(
    host: &SharedHost,
    lua: &Lua,
    spec: ListFilesSpec,
) -> Result<Value, String> {
    let inner = host.lock().map_err(|e| e.to_string())?;
    let paths = crate::sandbox::workspace_glob::resolve_globs(
        inner.sandbox.workspace_root(),
        &spec.patterns,
    )?;
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

