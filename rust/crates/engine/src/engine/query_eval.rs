//! Query match existence, counts, and capture spans.

use crate::ops::QuerySpec;
use tree_sitter::StreamingIterator;
use tree_sitter::{Query, QueryCursor, Tree};

use super::{
    node_for_byte_range, resolve_step_text, CaptureSpan, Engine, EngineError, QueryContext,
};

impl Engine {
    /// Collect all spans for `capture_name` (for editor derive / Lua query results).
    pub fn query_capture_spans(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        query_spec: &QuerySpec,
        capture_name: &str,
    ) -> Result<Vec<CaptureSpan>, EngineError> {
        let tree = self.parse_tree(source)?;
        self.query_capture_spans_on_tree(ctx, source, &tree, query_spec, capture_name)
    }

    /// Like [`Self::query_capture_spans`] but reuses an already-parsed tree.
    pub fn query_capture_spans_on_tree(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        tree: &Tree,
        query_spec: &QuerySpec,
        capture_name: &str,
    ) -> Result<Vec<CaptureSpan>, EngineError> {
        self.collect_all_capture_spans(ctx, source, tree, query_spec, capture_name)
    }

    /// One row per tree-sitter match; map of capture name → spans in that match.
    pub fn query_match_captures(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        query_spec: &QuerySpec,
    ) -> Result<Vec<std::collections::BTreeMap<String, Vec<CaptureSpan>>>, EngineError> {
        let tree = self.parse_tree(source)?;
        self.query_match_captures_on_tree(ctx, source, &tree, query_spec)
    }

    /// Like [`Self::query_match_captures`] but reuses an already-parsed tree.
    pub fn query_match_captures_on_tree(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        tree: &Tree,
        query_spec: &QuerySpec,
    ) -> Result<Vec<std::collections::BTreeMap<String, Vec<CaptureSpan>>>, EngineError> {
        self.collect_match_capture_rows(ctx, source, tree, query_spec)
    }

    /// True when the query chain produces at least one match on `source`.
    pub fn query_has_match(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        query_spec: &QuerySpec,
    ) -> Result<bool, EngineError> {
        let tree = self.parse_tree(source)?;
        self.query_has_match_on_tree(ctx, source, &tree, query_spec)
    }

    pub fn query_has_match_on_tree(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        tree: &Tree,
        query_spec: &QuerySpec,
    ) -> Result<bool, EngineError> {
        let count = self.count_query_matches(ctx, source, tree, query_spec, None)?;
        Ok(count > 0)
    }

    /// Number of matches (or capture hits when `capture_name` is set).
    pub fn query_match_count(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        query_spec: &QuerySpec,
        capture_name: Option<&str>,
    ) -> Result<usize, EngineError> {
        let tree = self.parse_tree(source)?;
        self.query_match_count_on_tree(ctx, source, &tree, query_spec, capture_name)
    }

    pub fn query_match_count_on_tree(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        tree: &Tree,
        query_spec: &QuerySpec,
        capture_name: Option<&str>,
    ) -> Result<usize, EngineError> {
        self.count_query_matches(ctx, source, tree, query_spec, capture_name)
    }

    fn count_query_matches(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        tree: &Tree,
        query_spec: &QuerySpec,
        capture_name: Option<&str>,
    ) -> Result<usize, EngineError> {
        if let Some(capture) = capture_name {
            let spans = self.collect_all_capture_spans(ctx, source, tree, query_spec, capture)?;
            return Ok(spans.len());
        }
        let steps = resolved_steps(ctx, query_spec)?;
        if steps.is_empty() {
            return Ok(0);
        }
        let language = self.adapter_language();
        let mut scope_spans: Vec<(usize, usize)> = vec![(0, source.len())];
        for (i, step_text) in steps.iter().enumerate() {
            let is_last = i == steps.len() - 1;
            if is_last {
                let query = Query::new(&language, step_text)
                    .map_err(|e| EngineError::Query(e.to_string()))?;
                let mut cursor = QueryCursor::new();
                let mut count = 0usize;
                for &(start, end) in &scope_spans {
                    let scope = node_for_byte_range(tree, start, end);
                    let mut matches = cursor.matches(&query, scope, source.as_bytes());
                    while matches.next().is_some() {
                        count += 1;
                    }
                }
                return Ok(count);
            }
            scope_spans =
                self.collect_match_root_spans(source, tree, &language, step_text, &scope_spans)?;
            if scope_spans.is_empty() {
                return Ok(0);
            }
        }
        Ok(0)
    }

    fn collect_all_capture_spans(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        tree: &Tree,
        query_spec: &QuerySpec,
        capture_name: &str,
    ) -> Result<Vec<CaptureSpan>, EngineError> {
        let steps = resolved_steps(ctx, query_spec)?;
        if steps.is_empty() {
            return Ok(vec![]);
        }
        let language = self.adapter_language();
        let mut scope_spans: Vec<(usize, usize)> = vec![(0, source.len())];
        for (i, step_text) in steps.iter().enumerate() {
            let is_last = i == steps.len() - 1;
            if is_last {
                let query = Query::new(&language, step_text)
                    .map_err(|e| EngineError::Query(e.to_string()))?;
                let capture_index = query
                    .capture_names()
                    .iter()
                    .position(|n| *n == capture_name)
                    .ok_or_else(|| EngineError::MissingCapture {
                        capture: capture_name.to_string(),
                    })?;
                let mut cursor = QueryCursor::new();
                let mut spans: Vec<CaptureSpan> = Vec::new();
                for &(start, end) in &scope_spans {
                    let scope = node_for_byte_range(tree, start, end);
                    let mut matches = cursor.matches(&query, scope, source.as_bytes());
                    while let Some(m) = matches.next() {
                        for cap in m.captures.iter() {
                            if cap.index as usize == capture_index {
                                let node = cap.node;
                                let span = CaptureSpan {
                                    start: node.start_byte(),
                                    end: node.end_byte(),
                                    is_block: node.kind() == "block",
                                    kind: node.kind().to_string(),
                                };
                                if !spans
                                    .iter()
                                    .any(|s| s.start == span.start && s.end == span.end)
                                {
                                    spans.push(span);
                                }
                            }
                        }
                    }
                }
                return Ok(spans);
            }
            scope_spans =
                self.collect_match_root_spans(source, tree, &language, step_text, &scope_spans)?;
            if scope_spans.is_empty() {
                return Ok(vec![]);
            }
        }
        Ok(vec![])
    }

    fn collect_match_capture_rows(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        tree: &Tree,
        query_spec: &QuerySpec,
    ) -> Result<Vec<std::collections::BTreeMap<String, Vec<CaptureSpan>>>, EngineError> {
        let steps = resolved_steps(ctx, query_spec)?;
        if steps.is_empty() {
            return Ok(vec![]);
        }
        let language = self.adapter_language();
        let mut scope_spans: Vec<(usize, usize)> = vec![(0, source.len())];
        for (i, step_text) in steps.iter().enumerate() {
            let is_last = i == steps.len() - 1;
            if is_last {
                let query = Query::new(&language, step_text)
                    .map_err(|e| EngineError::Query(e.to_string()))?;
                let names: Vec<String> = query
                    .capture_names()
                    .iter()
                    .map(|n| (*n).to_string())
                    .collect();
                let mut cursor = QueryCursor::new();
                let mut rows = Vec::new();
                for &(start, end) in &scope_spans {
                    let scope = node_for_byte_range(tree, start, end);
                    let mut matches = cursor.matches(&query, scope, source.as_bytes());
                    while let Some(m) = matches.next() {
                        let mut row: std::collections::BTreeMap<String, Vec<CaptureSpan>> =
                            std::collections::BTreeMap::new();
                        for cap in m.captures.iter() {
                            let name = names
                                .get(cap.index as usize)
                                .cloned()
                                .unwrap_or_else(|| format!("cap{}", cap.index));
                            if name.starts_with("__") {
                                continue;
                            }
                            let node = cap.node;
                            let span = CaptureSpan {
                                start: node.start_byte(),
                                end: node.end_byte(),
                                is_block: node.kind() == "block",
                                kind: node.kind().to_string(),
                            };
                            row.entry(name).or_default().push(span);
                        }
                        if !row.is_empty() {
                            rows.push(row);
                        }
                    }
                }
                return Ok(rows);
            }
            scope_spans =
                self.collect_match_root_spans(source, tree, &language, step_text, &scope_spans)?;
            if scope_spans.is_empty() {
                return Ok(vec![]);
            }
        }
        Ok(vec![])
    }
}

fn resolved_steps(
    ctx: &QueryContext<'_>,
    query_spec: &QuerySpec,
) -> Result<Vec<String>, EngineError> {
    match query_spec {
        QuerySpec::Single(s) => Ok(vec![resolve_step_text(ctx, s)?]),
        QuerySpec::Chain(v) => v.iter().map(|s| resolve_step_text(ctx, s)).collect(),
    }
}
