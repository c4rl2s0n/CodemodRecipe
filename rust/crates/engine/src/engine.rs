use std::path::Path;

use crate::adapter::LanguageAdapter;
use crate::query_chain::match_root_node;
use codemod_recipe_core::patch::{apply_patches, SourcePatch};
use codemod_recipe_yaml::model::{EditOp, EditStep, InsertAnchor, Recipe, Step};
use codemod_recipe_yaml::QuerySpec;
use thiserror::Error;
use tree_sitter::StreamingIterator;
use tree_sitter::{Node, Parser, Query, QueryCursor, Tree};

/// Paths used to resolve `query:` file references in recipes.
#[derive(Debug, Clone, Copy)]
pub struct QueryContext<'a> {
    pub recipe_file: Option<&'a Path>,
    pub codemod_root: &'a Path,
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("failed to parse yaml recipe: {0}")]
    RecipeParse(String),

    #[error("syntax errors present in file: {path}")]
    SyntaxError { path: String },

    #[error("failed to load language: {0}")]
    LanguageLoad(String),

    #[error("language not supported: {0}")]
    LanguageNotSupported(String),

    #[error("file type not supported: {0}")]
    FileTypeNotSupported(String),

    #[error("tree-sitter query error: {0}")]
    Query(String),

    #[error("capture not found in query: {capture}")]
    MissingCapture { capture: String },

    #[error("query matched no nodes for capture: {capture}")]
    NoMatch { capture: String },

    #[error("query matched multiple nodes for capture: {capture} (count={count})")]
    MultipleMatches { capture: String, count: usize },

    #[error(transparent)]
    Patch(#[from] codemod_recipe_core::patch::PatchError),
}

pub struct Engine {
    parser: Parser,
    adapter: Box<dyn LanguageAdapter>,
}

pub struct ApplyResult {
    pub modified: String,
    pub patches: Vec<SourcePatch>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct CaptureSpan {
    pub start: usize,
    pub end: usize,
    pub is_block: bool,
}

impl Engine {
    pub fn new(adapter: Box<dyn LanguageAdapter>) -> Result<Self, EngineError> {
        let mut parser = Parser::new();
        parser
            .set_language(&adapter.language())
            .map_err(|e| EngineError::Query(format!("set_language failed: {e:?}")))?;
        Ok(Self { parser, adapter })
    }

    pub(crate) fn adapter_language(&self) -> tree_sitter::Language {
        self.adapter.language()
    }

    /// Collect patches for a single edit op against `source`.
    pub fn collect_patches_for_single_op(
        &mut self,
        ctx: &QueryContext<'_>,
        op: &EditOp,
        source: &str,
    ) -> Result<Vec<SourcePatch>, EngineError> {
        let tree = self.parse_tree(source)?;
        match op {
            EditOp::Insert(insert) => {
                let span = self.resolve_capture_span(
                    ctx,
                    source,
                    &tree,
                    &insert.query,
                    &insert.capture,
                    true,
                )?;
                let Some(span) = span else {
                    return Err(EngineError::NoMatch {
                        capture: insert.capture.clone(),
                    });
                };
                let offset = match insert.anchor {
                    InsertAnchor::Start => span.start,
                    InsertAnchor::End => crate::span::insert_offset_at_anchor_end(
                        source,
                        span.start,
                        span.end,
                        span.is_block,
                    ),
                };
                Ok(vec![SourcePatch::new(offset, offset, insert.text.clone())])
            }
            EditOp::Replace(replace) => {
                let span = self.resolve_capture_span(
                    ctx,
                    source,
                    &tree,
                    &replace.query,
                    &replace.capture,
                    true,
                )?;
                let Some(span) = span else {
                    return Ok(vec![]);
                };
                let node = node_for_byte_range(&tree, span.start, span.end);
                let (start, end) = self.adapter.expand_remove_span(
                    source,
                    &tree,
                    node,
                    span.start,
                    span.end,
                    replace.include_leading_trivia,
                );
                let current = &source[start..end];
                if whitespace_normalized(current) == whitespace_normalized(&replace.text) {
                    return Ok(vec![]);
                }
                Ok(vec![SourcePatch::new(start, end, replace.text.clone())])
            }
            EditOp::Remove(remove) => {
                let span = self.resolve_capture_span(
                    ctx,
                    source,
                    &tree,
                    &remove.query,
                    &remove.capture,
                    true,
                )?;
                let Some(span) = span else {
                    return Ok(vec![]);
                };
                let node = node_for_byte_range(&tree, span.start, span.end);
                let (start, end) = self.adapter.expand_remove_span(
                    source,
                    &tree,
                    node,
                    span.start,
                    span.end,
                    remove.include_leading_trivia,
                );
                Ok(vec![SourcePatch::new(start, end, "")])
            }
            EditOp::Unknown(_, _) => Ok(vec![]),
        }
    }

    /// Collect patches for all ops in an edit against one `source` (non-sequential).
    /// Prefer [`Self::apply_edit_ops_sequential`] when later ops depend on earlier ones.
    pub fn collect_patches_for_edit(
        &mut self,
        ctx: &QueryContext<'_>,
        edit: &EditStep,
        source: &str,
    ) -> Result<Vec<SourcePatch>, EngineError> {
        let mut patches: Vec<SourcePatch> = Vec::new();
        for op in &edit.ops {
            patches.extend(self.collect_patches_for_single_op(ctx, op, source)?);
        }
        Ok(patches)
    }

    /// Apply each op in order: resolve against current text, apply patches, continue.
    pub fn apply_edit_ops_sequential(
        &mut self,
        ctx: &QueryContext<'_>,
        edit: &EditStep,
        source: &str,
    ) -> Result<String, EngineError> {
        let mut current = source.to_string();
        for op in &edit.ops {
            let patches = self.collect_patches_for_single_op(ctx, op, &current)?;
            if !patches.is_empty() {
                current = apply_patches(&current, &patches)?;
            }
        }
        Ok(current)
    }

    /// Apply all edit steps for `file_path` sequentially against evolving source.
    pub fn apply_recipe_to_source(
        &mut self,
        ctx: &QueryContext<'_>,
        recipe: &Recipe,
        file_path: &str,
        source: &str,
    ) -> Result<ApplyResult, EngineError> {
        let mut current = source.to_string();
        for step in &recipe.steps {
            let Step::Edit(edit) = step else { continue };
            if edit.path != file_path {
                continue;
            }
            current = self.apply_edit_ops_sequential(ctx, edit, &current)?;
        }
        let patches = if current == source {
            vec![]
        } else {
            vec![SourcePatch::new(0, source.len(), current.clone())]
        };
        Ok(ApplyResult {
            modified: current,
            patches,
        })
    }

    pub(crate) fn parse_tree(&mut self, source: &str) -> Result<Tree, EngineError> {
        let tree = self
            .parser
            .parse(source, None)
            .ok_or_else(|| EngineError::SyntaxError {
                path: "<memory>".to_string(),
            })?;
        if tree.root_node().has_error() {
            return Err(EngineError::SyntaxError {
                path: "<memory>".to_string(),
            });
        }
        Ok(tree)
    }

    fn resolve_capture_span(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        tree: &Tree,
        query_spec: &QuerySpec,
        capture_name: &str,
        fail_on_multiple: bool,
    ) -> Result<Option<CaptureSpan>, EngineError> {
        let steps: Vec<String> = match query_spec {
            QuerySpec::Single(s) => vec![resolve_step_text(ctx, s)?],
            QuerySpec::Chain(v) => v
                .iter()
                .map(|s| resolve_step_text(ctx, s))
                .collect::<Result<Vec<_>, _>>()?,
        };

        if steps.is_empty() {
            return Ok(None);
        }

        let language = self.adapter.language();
        let mut scope_spans: Vec<(usize, usize)> = vec![(0, source.len())];

        for (i, step_text) in steps.iter().enumerate() {
            let is_last = i == steps.len() - 1;
            if is_last {
                return self.collect_capture_spans(
                    source,
                    tree,
                    &language,
                    step_text,
                    capture_name,
                    &scope_spans,
                    fail_on_multiple,
                );
            }
            scope_spans = self.collect_match_root_spans(
                source,
                tree,
                &language,
                step_text,
                &scope_spans,
            )?;
            if scope_spans.is_empty() {
                return Ok(None);
            }
        }
        Ok(None)
    }

    pub(crate) fn collect_match_root_spans(
        &self,
        source: &str,
        tree: &Tree,
        language: &tree_sitter::Language,
        query_text: &str,
        scope_spans: &[(usize, usize)],
    ) -> Result<Vec<(usize, usize)>, EngineError> {
        let query =
            Query::new(language, query_text).map_err(|e| EngineError::Query(e.to_string()))?;
        let mut cursor = QueryCursor::new();
        let mut roots = Vec::new();
        for &(start, end) in scope_spans {
            let scope = node_for_byte_range(tree, start, end);
            let mut matches = cursor.matches(&query, scope, source.as_bytes());
            while let Some(m) = matches.next() {
                if let Some(root) = match_root_node(m) {
                    let span = (root.start_byte(), root.end_byte());
                    if !roots.contains(&span) {
                        roots.push(span);
                    }
                }
            }
        }
        Ok(roots)
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_capture_spans(
        &self,
        source: &str,
        tree: &Tree,
        language: &tree_sitter::Language,
        query_text: &str,
        capture_name: &str,
        scope_spans: &[(usize, usize)],
        fail_on_multiple: bool,
    ) -> Result<Option<CaptureSpan>, EngineError> {
        let query =
            Query::new(language, query_text).map_err(|e| EngineError::Query(e.to_string()))?;
        let capture_index = query
            .capture_names()
            .iter()
            .position(|n| *n == capture_name)
            .ok_or_else(|| EngineError::MissingCapture {
                capture: capture_name.to_string(),
            })?;

        let mut cursor = QueryCursor::new();
        let mut spans: Vec<CaptureSpan> = Vec::new();
        for &(start, end) in scope_spans {
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
                        if !spans.iter().any(|s| s.start == span.start && s.end == span.end) {
                            spans.push(span);
                        }
                    }
                }
            }
        }

        match spans.len() {
            0 => Ok(None),
            1 => Ok(Some(spans[0])),
            n if fail_on_multiple => Err(EngineError::MultipleMatches {
                capture: capture_name.to_string(),
                count: n,
            }),
            _ => Ok(spans.into_iter().next()),
        }
    }
}

pub(crate) fn node_for_byte_range(tree: &Tree, start: usize, end: usize) -> Node<'_> {
    tree.root_node()
        .named_descendant_for_byte_range(start, end.max(start + 1))
        .or_else(|| tree.root_node().descendant_for_byte_range(start, end.max(start + 1)))
        .unwrap_or_else(|| tree.root_node())
}

pub(crate) fn resolve_step_text(ctx: &QueryContext<'_>, raw: &str) -> Result<String, EngineError> {
    crate::query::resolve_query_source(raw, ctx.recipe_file, ctx.codemod_root)
}

fn whitespace_normalized(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn parse_recipe_yaml(yaml_text: &str) -> Result<Recipe, EngineError> {
    serde_yaml::from_str::<Recipe>(yaml_text).map_err(|e| EngineError::RecipeParse(e.to_string()))
}
