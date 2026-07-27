//! Guard and `let` query evaluation (match existence, extract capture values).

use codemod_recipe_yaml::let_binding::{
    LetBinding, LetExtract, LetOnManyMatches, LetOnNoMatch,
};
use codemod_recipe_yaml::QuerySpec;
use tree_sitter::StreamingIterator;
use tree_sitter::{Query, QueryCursor, Tree};

use super::engine::{CaptureSpan, Engine, EngineError, QueryContext, node_for_byte_range, resolve_step_text};

impl Engine {
    /// True when the query chain produces at least one match on `source`.
    pub fn query_has_match(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        query_spec: &QuerySpec,
    ) -> Result<bool, EngineError> {
        let tree = self.parse_tree(source)?;
        let count = self.count_query_matches(ctx, source, &tree, query_spec, None)?;
        Ok(count > 0)
    }

    /// Evaluate one `let` binding against `source`.
    pub fn evaluate_let_binding(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        binding: &LetBinding,
    ) -> Result<String, EngineError> {
        let query = binding.query.as_ref().ok_or_else(|| {
            EngineError::Query(format!(
                "let binding '{}' requires query unless only `as` is used",
                binding.name
            ))
        })?;
        let tree = self.parse_tree(source)?;
        match binding.extract {
            LetExtract::Exists => {
                let capture = binding.capture.as_deref();
                let count =
                    self.count_query_matches(ctx, source, &tree, query, capture)?;
                Ok(if count > 0 {
                    "true".to_string()
                } else {
                    "false".to_string()
                })
            }
            LetExtract::Count => {
                let capture = binding.capture.as_deref();
                let count =
                    self.count_query_matches(ctx, source, &tree, query, capture)?;
                Ok(count.to_string())
            }
            LetExtract::Text | LetExtract::Kind => {
                let capture = binding.capture.as_deref().ok_or_else(|| {
                    EngineError::Query(format!(
                        "let binding '{}' requires capture for extract {:?}",
                        binding.name, binding.extract
                    ))
                })?;
                self.extract_single_capture_value(
                    ctx,
                    source,
                    &tree,
                    query,
                    capture,
                    binding,
                )
            }
        }
    }

    fn extract_single_capture_value(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        tree: &Tree,
        query_spec: &QuerySpec,
        capture_name: &str,
        binding: &LetBinding,
    ) -> Result<String, EngineError> {
        let spans =
            self.collect_all_capture_spans(ctx, source, tree, query_spec, capture_name)?;
        if spans.is_empty() {
            return match binding.on_no_match {
                LetOnNoMatch::Error => Err(EngineError::NoMatch {
                    capture: capture_name.to_string(),
                }),
                LetOnNoMatch::UseEmpty => Ok(String::new()),
            };
        }
        match spans.len() {
            1 => node_projection(source, tree, &spans[0], binding.extract),
            _ => match binding.on_many_matches {
                LetOnManyMatches::Error => Err(EngineError::MultipleMatches {
                    capture: capture_name.to_string(),
                    count: spans.len(),
                }),
                LetOnManyMatches::First => {
                    node_projection(source, tree, &spans[0], binding.extract)
                }
                LetOnManyMatches::Join => {
                    let sep = binding.join.as_deref().unwrap_or(",");
                    let parts: Vec<String> = spans
                        .iter()
                        .map(|s| node_projection(source, tree, s, binding.extract))
                        .collect::<Result<_, _>>()?;
                    Ok(parts.join(sep))
                }
            },
        }
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
            scope_spans = self.collect_match_root_spans(
                source,
                tree,
                &language,
                step_text,
                &scope_spans,
            )?;
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
            scope_spans = self.collect_match_root_spans(
                source,
                tree,
                &language,
                step_text,
                &scope_spans,
            )?;
            if scope_spans.is_empty() {
                return Ok(vec![]);
            }
        }
        Ok(vec![])
    }
}

fn resolved_steps(ctx: &QueryContext<'_>, query_spec: &QuerySpec) -> Result<Vec<String>, EngineError> {
    match query_spec {
        QuerySpec::Single(s) => Ok(vec![resolve_step_text(ctx, s)?]),
        QuerySpec::Chain(v) => v
            .iter()
            .map(|s| resolve_step_text(ctx, s))
            .collect(),
    }
}

fn node_projection(
    source: &str,
    tree: &Tree,
    span: &CaptureSpan,
    extract: LetExtract,
) -> Result<String, EngineError> {
    match extract {
        LetExtract::Text => Ok(source[span.start..span.end].to_string()),
        LetExtract::Kind => {
            let node = node_for_byte_range(tree, span.start, span.end);
            Ok(node.kind().to_string())
        }
        LetExtract::Exists | LetExtract::Count => Err(EngineError::Query(
            "internal: node_projection used with exists/count".to_string(),
        )),
    }
}
