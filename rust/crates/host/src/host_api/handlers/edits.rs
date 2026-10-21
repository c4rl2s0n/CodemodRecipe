use codemod_recipe_engine::ops::{EditOp, InsertAnchor, InsertOp, RemoveOp, ReplaceOp};
use codemod_recipe_lua::host_api::specs::{Anchor, InsertSpec, RemoveSpec, ReplaceSpec};
use codemod_recipe_lua::model::{TranscriptEvent, TranscriptStats};
use codemod_recipe_lua_macros::host_method;
use mlua::{Lua, Value};
use serde_json::json;

use crate::host_api::context::SharedHost;
use crate::host_api::facets::RecipeSession;
use super::helpers::{
    apply_source_op, capture_texts, count_lines, query_label, query_spec_to_engine, trunc_text,
};

#[host_method(
    name = "insert",
    description = "Insert text at a tree-sitter capture (`path`, `query`, `capture`, `text`).",
    spec = InsertSpec,
    return_type = Nil,
)]
pub(crate) fn do_insert(host: &SharedHost, _lua: &Lua, spec: InsertSpec) -> Result<Value, String> {
    RecipeSession::with_host(host, |mut s| {
        s.bump_ops()?;
        let query = query_spec_to_engine(&spec.query, s.queries())?;
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
        let next = apply_source_op(s.host_mut(), &path, spec.language.as_deref(), &op)?;
        s.apply_edit(&path, |_| Ok(next))?;
        let mut payload = json!({
            "capture": capture,
            "anchor": anchor_label,
            "text": trunc_text(&text),
        });
        if let Some(q) = query_detail {
            payload["query"] = json!(q);
        }
        s.transcript().push(TranscriptEvent::edit(
            "insert",
            path,
            TranscriptStats {
                added_lines: Some(count_lines(&text)),
                removed_lines: None,
            },
            payload,
        ));
        Ok(Value::Nil)
    })
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
    RecipeSession::with_host(host, |mut s| {
        s.bump_ops()?;
        let query = query_spec_to_engine(&spec.query, s.queries())?;
        let query_detail = query_label(&spec.query);
        let capture = spec.capture.clone();
        let text = spec.text.clone();
        let removed = capture_texts(
            s.host_mut(),
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
        let next = apply_source_op(s.host_mut(), &path, spec.language.as_deref(), &op)?;
        s.apply_edit(&path, |_| Ok(next))?;
        let mut payload = json!({
            "capture": capture,
            "text": trunc_text(&text),
            "removedText": trunc_text(&removed_joined),
        });
        if let Some(q) = query_detail {
            payload["query"] = json!(q);
        }
        s.transcript().push(TranscriptEvent::edit(
            "replace",
            path,
            TranscriptStats {
                added_lines: Some(count_lines(&text) * match_count),
                removed_lines: Some(removed.iter().map(|s| count_lines(s)).sum()),
            },
            payload,
        ));
        Ok(Value::Nil)
    })
}

#[host_method(
    name = "remove",
    description = "Remove a capture span.",
    spec = RemoveSpec,
    return_type = Nil,
)]
pub(crate) fn do_remove(host: &SharedHost, _lua: &Lua, spec: RemoveSpec) -> Result<Value, String> {
    RecipeSession::with_host(host, |mut s| {
        s.bump_ops()?;
        let query = query_spec_to_engine(&spec.query, s.queries())?;
        let query_detail = query_label(&spec.query);
        let capture = spec.capture.clone();
        let removed = capture_texts(
            s.host_mut(),
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
        let next = apply_source_op(s.host_mut(), &path, spec.language.as_deref(), &op)?;
        s.apply_edit(&path, |_| Ok(next))?;
        let mut payload = json!({
            "capture": capture,
            "removedText": trunc_text(&removed_joined),
        });
        if let Some(q) = query_detail {
            payload["query"] = json!(q);
        }
        s.transcript().push(TranscriptEvent::edit(
            "remove",
            path,
            TranscriptStats {
                added_lines: None,
                removed_lines: Some(removed.iter().map(|s| count_lines(s)).sum()),
            },
            payload,
        ));
        Ok(Value::Nil)
    })
}
