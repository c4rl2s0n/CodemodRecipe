use std::collections::BTreeMap;

use codemod_recipe_engine::engine::{CaptureSpan, Engine, QueryContext};
use codemod_recipe_engine::ops::{EditOp, QuerySpec as EngineQuerySpec};
use codemod_recipe_lua::host_api::specs::{QuerySpec as RawQuerySpec, QueryTableSpec};
use codemod_recipe_lua::model::TranscriptEvent;
use mlua::{Lua, Table, Value};
use serde_json::json;

use crate::edit_session::post_execution::run_post_execution_command;
use crate::host_api::context::{bump_ops, SharedHost};

pub(crate) const TRANSCRIPT_TEXT_LIMIT: usize = 4096;

pub(crate) fn count_lines(s: &str) -> i64 {
    if s.is_empty() {
        0
    } else {
        s.lines().count() as i64
    }
}

pub(crate) fn trunc_text(s: &str) -> String {
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

pub(crate) fn query_label(query: &RawQuerySpec) -> Option<String> {
    let steps: Vec<&str> = query.iter().collect();
    if steps.is_empty() {
        None
    } else if steps.len() == 1 {
        Some(trunc_text(steps[0]))
    } else {
        Some(format!("{} steps", steps.len()))
    }
}

pub(crate) fn expect_match(host: &SharedHost, spec: &QueryTableSpec, want: bool) -> Result<Value, String> {
    let n = query_count(host, spec)?;
    if want && n == 0 {
        return Err("expected query to match".to_string());
    }
    if !want && n > 0 {
        return Err("expected query not to match".to_string());
    }
    Ok(Value::Nil)
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

pub(crate) fn query_spec_to_engine(
    spec: &RawQuerySpec,
    queries: &BTreeMap<String, String>,
) -> Result<EngineQuerySpec, String> {
    let mut steps: Vec<String> = spec
        .iter()
        .map(|raw| queries.get(raw).cloned().unwrap_or_else(|| raw.to_string()))
        .collect();
    // Resolve library aliases, then own as core/engine QuerySpec (D-016).
    if steps.len() <= 1 {
        Ok(EngineQuerySpec::single(steps.pop().unwrap_or_default()))
    } else {
        Ok(EngineQuerySpec::Chain(steps))
    }
}

pub(crate) fn apply_source_op(
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

pub(crate) fn capture_texts(
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

pub(crate) fn query_count(host: &SharedHost, spec: &QueryTableSpec) -> Result<usize, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    let query = query_spec_to_engine(&spec.query, &inner.queries)?;
    let lang_id = inner
        .language_registry
        .resolve_language_id(spec.language.as_deref(), &spec.path)
        .map_err(|e| e.to_string())?;
    let recipe_path = inner.recipe_path.clone();
    let codemod = inner.codemod_root.clone();
    let source = inner.current_content(&spec.path)?;
    let cached = inner.cached_tree(&spec.path, &source, spec.language.as_deref())?;
    let qctx = QueryContext {
        recipe_file: recipe_path.as_deref(),
        codemod_root: &codemod,
    };
    let engine = inner
        .language_registry
        .get(&lang_id)
        .map_err(|e| e.to_string())?;
    engine
        .query_match_count_on_tree(
            &qctx,
            &cached.source,
            &cached.tree,
            &query,
            spec.capture.as_deref(),
        )
        .map_err(|e| e.to_string())
}

pub(crate) fn string_map_to_table(lua: &Lua, map: &BTreeMap<String, String>) -> Result<Value, String> {
    let t = lua.create_table().map_err(|e| e.to_string())?;
    for (k, v) in map {
        t.set(k.as_str(), v.as_str()).map_err(|e| e.to_string())?;
    }
    Ok(Value::Table(t))
}

pub(crate) fn hit_table(lua: &Lua, source: &str, span: &CaptureSpan) -> Result<Table, String> {
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
