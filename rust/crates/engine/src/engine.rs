use std::path::Path;

use crate::adapter::LanguageAdapter;
use codemod_recipe_core::patch::{apply_patches, SourcePatch};
use codemod_recipe_yaml::model::{EditOp, InsertAnchor, Recipe, Step};
use thiserror::Error;
use tree_sitter::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

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
    adapter: crate::adapter::DartAdapter,
}

pub struct ApplyResult {
    pub modified: String,
    pub patches: Vec<SourcePatch>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CaptureSpan {
    start: usize,
    end: usize,
    is_block: bool,
}

impl Engine {
    pub fn new_dart() -> Result<Self, EngineError> {
        let mut parser = Parser::new();
        parser
            .set_language(&crate::dart::language())
            .map_err(|e| EngineError::Query(format!("set_language failed: {e:?}")))?;
        Ok(Self {
            parser,
            adapter: crate::adapter::DartAdapter,
        })
    }

    pub fn collect_patches_for_source(
        &mut self,
        ctx: &QueryContext<'_>,
        recipe: &Recipe,
        file_path: &str,
        source: &str,
    ) -> Result<Vec<SourcePatch>, EngineError> {
        let mut patches: Vec<SourcePatch> = Vec::new();

        for step in &recipe.steps {
            let Step::Edit(edit) = step else { continue };
            if edit.path != file_path {
                continue;
            }

            for op in &edit.ops {
                match op {
                    EditOp::Insert(insert) => {
                        let span = self.resolve_single_capture(
                            ctx,
                            source,
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
                        patches.push(SourcePatch::new(offset, offset, insert.text.clone()));
                    }
                    EditOp::Replace(replace) => {
                        let span = self.resolve_single_capture(
                            ctx,
                            source,
                            &replace.query,
                            &replace.capture,
                            true,
                        )?;
                        let Some(span) = span else {
                            continue;
                        };
                        let (start, end) = self.adapter.expand_remove_span(
                            source,
                            span.start,
                            span.end,
                            replace.include_leading_trivia,
                        );
                        let current = &source[start..end];
                        if whitespace_normalized(current) == whitespace_normalized(&replace.text) {
                            continue;
                        }
                        patches.push(SourcePatch::new(start, end, replace.text.clone()));
                    }
                    EditOp::Remove(remove) => {
                        let span = self.resolve_single_capture(
                            ctx,
                            source,
                            &remove.query,
                            &remove.capture,
                            true,
                        )?;
                        let Some(span) = span else {
                            continue;
                        };
                        let (start, end) = self.adapter.expand_remove_span(
                            source,
                            span.start,
                            span.end,
                            remove.include_leading_trivia,
                        );
                        patches.push(SourcePatch::new(start, end, ""));
                    }
                    EditOp::Unknown(_, _) => {}
                }
            }
        }

        Ok(patches)
    }

    pub fn apply_recipe_to_source(
        &mut self,
        ctx: &QueryContext<'_>,
        recipe: &Recipe,
        file_path: &str,
        source: &str,
    ) -> Result<ApplyResult, EngineError> {
        let patches = self.collect_patches_for_source(ctx, recipe, file_path, source)?;
        let modified = apply_patches(source, &patches)?;
        Ok(ApplyResult { modified, patches })
    }

    fn resolve_single_capture(
        &mut self,
        ctx: &QueryContext<'_>,
        source: &str,
        query_source: &str,
        capture_name: &str,
        fail_on_multiple: bool,
    ) -> Result<Option<CaptureSpan>, EngineError> {
        let query_source = crate::query::resolve_query_source(
            query_source,
            ctx.recipe_file,
            ctx.codemod_root,
        )?;

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

        let language = self.adapter.language();
        let query =
            Query::new(&language, &query_source).map_err(|e| EngineError::Query(e.to_string()))?;

        let capture_index = query
            .capture_names()
            .iter()
            .position(|n| *n == capture_name)
            .ok_or_else(|| EngineError::MissingCapture {
                capture: capture_name.to_string(),
            })?;

        let mut cursor = QueryCursor::new();
        let mut matches_iter = cursor.matches(&query, tree.root_node(), source.as_bytes());

        let mut spans: Vec<CaptureSpan> = Vec::new();
        while let Some(m) = matches_iter.next() {
            for cap in m.captures.iter() {
                if cap.index as usize == capture_index {
                    let node = cap.node;
                    spans.push(CaptureSpan {
                        start: node.start_byte(),
                        end: node.end_byte(),
                        is_block: node.kind() == "block",
                    });
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

fn whitespace_normalized(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn parse_recipe_yaml(yaml_text: &str) -> Result<Recipe, EngineError> {
    serde_yaml::from_str::<Recipe>(yaml_text).map_err(|e| EngineError::RecipeParse(e.to_string()))
}
