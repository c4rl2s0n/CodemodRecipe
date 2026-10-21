use codemod_recipe_engine::engine::QueryContext;
use codemod_recipe_lua::host_api::specs::{
    QueryCaptureSpec, QueryHit, QueryMatchRow, QueryTableSpec,
};
use codemod_recipe_lua::model::TranscriptEvent;
use codemod_recipe_lua_macros::host_method;
use mlua::{Lua, Value};
use serde_json::json;

use crate::host_api::context::SharedHost;
use crate::host_api::facets::QueryIntel;
use super::helpers::{
    expect_match, hit_table, query_count, query_spec_to_engine, trunc_text,
};

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
    QueryIntel::with_host(host, |mut q| {
        let query = query_spec_to_engine(&spec.query, q.queries())?;
        let lang_id = q
            .language_registry()
            .resolve_language_id(spec.language.as_deref(), &spec.path)
            .map_err(|e| e.to_string())?;
        let recipe_path = q.recipe_path().cloned();
        let codemod = q.codemod_root().to_path_buf();
        let source = q.current_content(&spec.path)?;
        let cached = q.cached_tree(&spec.path, &source, spec.language.as_deref())?;
        let qctx = QueryContext {
            recipe_file: recipe_path.as_deref(),
            codemod_root: &codemod,
        };
        let engine = q
            .language_registry_mut()
            .get(&lang_id)
            .map_err(|e| e.to_string())?;
        let spans = engine
            .query_capture_spans_on_tree(&qctx, &cached.source, &cached.tree, &query, capture)
            .map_err(|e| e.to_string())?;
        let out = lua.create_table().map_err(|e| e.to_string())?;
        for (i, span) in spans.iter().enumerate() {
            let row = hit_table(lua, &cached.source, span)?;
            out.set(i + 1, row).map_err(|e| e.to_string())?;
        }
        let first_snippet = spans
            .first()
            .map(|span| trunc_text(&cached.source[span.start..span.end]));
        q.transcript().push(TranscriptEvent::with_payload(
            "query",
            format!("{}#{} n={}", spec.path, capture, spans.len()),
            json!({
                "path": spec.path,
                "capture": capture,
                "matchCount": spans.len(),
                "firstMatch": first_snippet,
                "cacheHit": cached.cache_hit,
            }),
        ));
        Ok(Value::Table(out))
    })
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
    QueryIntel::with_host(host, |mut q| {
        let query = query_spec_to_engine(&spec.query, q.queries())?;
        let lang_id = q
            .language_registry()
            .resolve_language_id(spec.language.as_deref(), &spec.path)
            .map_err(|e| e.to_string())?;
        let recipe_path = q.recipe_path().cloned();
        let codemod = q.codemod_root().to_path_buf();
        let source = q.current_content(&spec.path)?;
        let cached = q.cached_tree(&spec.path, &source, spec.language.as_deref())?;
        let qctx = QueryContext {
            recipe_file: recipe_path.as_deref(),
            codemod_root: &codemod,
        };
        let engine = q
            .language_registry_mut()
            .get(&lang_id)
            .map_err(|e| e.to_string())?;
        let rows = engine
            .query_match_captures_on_tree(&qctx, &cached.source, &cached.tree, &query)
            .map_err(|e| e.to_string())?;
        let out = lua.create_table().map_err(|e| e.to_string())?;
        for (i, row) in rows.iter().enumerate() {
            let entry = lua.create_table().map_err(|e| e.to_string())?;
            for (name, spans) in row {
                let value = match spans.as_slice() {
                    [] => Value::Nil,
                    [one] => Value::Table(hit_table(lua, &cached.source, one)?),
                    many => {
                        let arr = lua.create_table().map_err(|e| e.to_string())?;
                        for (j, span) in many.iter().enumerate() {
                            arr.set(j + 1, hit_table(lua, &cached.source, span)?)
                                .map_err(|e| e.to_string())?;
                        }
                        Value::Table(arr)
                    }
                };
                entry.set(name.as_str(), value).map_err(|e| e.to_string())?;
            }
            out.set(i + 1, entry).map_err(|e| e.to_string())?;
        }
        q.transcript().push(TranscriptEvent::with_payload(
            "queryMatches",
            format!("{} n={}", spec.path, rows.len()),
            json!({
                "path": spec.path,
                "matchCount": rows.len(),
                "cacheHit": cached.cache_hit,
            }),
        ));
        Ok(Value::Table(out))
    })
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
