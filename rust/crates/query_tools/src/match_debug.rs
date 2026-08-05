use serde::Serialize;
use tree_sitter::{Language, Parser, Query, QueryCursor, StreamingIterator, Tree};

use crate::instrument::instrument_query;
use crate::position::byte_to_position;
use crate::QueryToolsError;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NodeSpan {
    pub kind: String,
    pub start: u32,
    pub end: u32,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CaptureInfo {
    pub name: String,
    pub kind: String,
    pub start: u32,
    pub end: u32,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    /// Nesting depth for `__layer_*` captures; author captures use 0.
    pub depth: u32,
    pub is_layer: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_start: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_end: Option<u32>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DebugMatch {
    pub root: NodeSpan,
    pub captures: Vec<CaptureInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DebugQueryResult {
    pub has_error: bool,
    pub match_count: usize,
    pub matches: Vec<DebugMatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrumented_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub root_sexp: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DebugOptions {
    pub instrument: bool,
    pub include_sexp: bool,
    pub max_text_len: usize,
    pub max_sexp_chars: usize,
}

impl Default for DebugOptions {
    fn default() -> Self {
        Self {
            instrument: true,
            include_sexp: false,
            max_text_len: 200,
            max_sexp_chars: 4000,
        }
    }
}

/// Run a tree-sitter query and return all matches with capture metadata.
pub fn debug_query(
    language: &Language,
    source: &str,
    query_text: &str,
    opts: &DebugOptions,
) -> Result<DebugQueryResult, QueryToolsError> {
    let mut parser = Parser::new();
    parser
        .set_language(language)
        .map_err(|e| QueryToolsError::Query(format!("set_language: {e:?}")))?;
    let tree = parser
        .parse(source, None)
        .ok_or(QueryToolsError::ParseFailed)?;
    let has_error = tree.root_node().has_error();

    let (run_query, layer_meta) = if opts.instrument {
        match instrument_query(query_text) {
            Ok(inst) => {
                let map: std::collections::HashMap<String, (u32, usize, usize)> = inst
                    .layer_ranges
                    .iter()
                    .map(|l| {
                        (
                            l.name.clone(),
                            (l.depth, l.query_start, l.query_end),
                        )
                    })
                    .collect();
                let q = inst.query.clone();
                (inst.query, Some((map, q)))
            }
            Err(_) => (query_text.to_string(), None),
        }
    } else {
        (query_text.to_string(), None)
    };

    let query = Query::new(language, &run_query)
        .map_err(|e| QueryToolsError::Query(e.to_string()))?;

    let mut cursor = QueryCursor::new();
    let mut matches_out = Vec::new();
    let mut matches_iter = cursor.matches(&query, tree.root_node(), source.as_bytes());
    while let Some(m) = matches_iter.next() {
        let mut captures = Vec::new();
        let mut nodes_for_root = Vec::new();
        for cap in m.captures.iter() {
            let name = query.capture_names()[cap.index as usize].to_string();
            let node = cap.node;
            nodes_for_root.push(node);
            let (is_layer, depth, qstart, qend) = if name.starts_with("__layer_") {
                if let Some((map, _)) = &layer_meta {
                    if let Some(&(d, qs, qe)) = map.get(&name) {
                        (true, d, Some(qs as u32), Some(qe as u32))
                    } else {
                        (true, 0, None, None)
                    }
                } else {
                    (true, 0, None, None)
                }
            } else {
                (false, 0, None, None)
            };
            captures.push(span_to_capture(
                source,
                &name,
                node,
                opts.max_text_len,
                depth,
                is_layer,
                qstart,
                qend,
            ));
        }
        let root_node = lowest_common_ancestor(&nodes_for_root).unwrap_or(tree.root_node());
        matches_out.push(DebugMatch {
            root: node_to_span(source, root_node, opts.max_text_len),
            captures,
        });
    }

    let root_sexp = if opts.include_sexp {
        Some(truncate(
            &tree.root_node().to_sexp(),
            opts.max_sexp_chars,
        ))
    } else {
        None
    };

    let instrumented_query = layer_meta.map(|(_, q)| q);
    let match_count = matches_out.len();

    Ok(DebugQueryResult {
        has_error,
        match_count,
        matches: matches_out,
        instrumented_query,
        root_sexp,
    })
}

/// Parse source with `language` (helper for callers that only have Language).
pub fn parse_tree(language: &Language, source: &str) -> Result<Tree, QueryToolsError> {
    let mut parser = Parser::new();
    parser
        .set_language(language)
        .map_err(|e| QueryToolsError::Query(format!("set_language: {e:?}")))?;
    parser
        .parse(source, None)
        .ok_or(QueryToolsError::ParseFailed)
}

fn span_to_capture(
    source: &str,
    name: &str,
    node: tree_sitter::Node<'_>,
    max_text: usize,
    depth: u32,
    is_layer: bool,
    query_start: Option<u32>,
    query_end: Option<u32>,
) -> CaptureInfo {
    let start = node.start_byte();
    let end = node.end_byte();
    let sp = byte_to_position(source, start);
    let ep = byte_to_position(source, end);
    CaptureInfo {
        name: name.to_string(),
        kind: node.kind().to_string(),
        start: start as u32,
        end: end as u32,
        start_line: sp.line,
        start_column: sp.column,
        end_line: ep.line,
        end_column: ep.column,
        text: clip_text(source, start, end, max_text),
        depth,
        is_layer,
        query_start,
        query_end,
    }
}

fn node_to_span(source: &str, node: tree_sitter::Node<'_>, max_text: usize) -> NodeSpan {
    let start = node.start_byte();
    let end = node.end_byte();
    let sp = byte_to_position(source, start);
    let ep = byte_to_position(source, end);
    NodeSpan {
        kind: node.kind().to_string(),
        start: start as u32,
        end: end as u32,
        start_line: sp.line,
        start_column: sp.column,
        end_line: ep.line,
        end_column: ep.column,
        text: clip_text(source, start, end, max_text),
    }
}

fn clip_text(source: &str, start: usize, end: usize, max: usize) -> Option<String> {
    let s = source.get(start..end)?;
    if s.len() <= max {
        Some(s.to_string())
    } else {
        Some(format!("{}…", &s[..max]))
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max])
    }
}

fn node_contains(ancestor: tree_sitter::Node<'_>, descendant: tree_sitter::Node<'_>) -> bool {
    if ancestor.id() == descendant.id() {
        return true;
    }
    ancestor.start_byte() <= descendant.start_byte() && ancestor.end_byte() >= descendant.end_byte()
}

fn lowest_common_ancestor<'a>(
    nodes: &[tree_sitter::Node<'a>],
) -> Option<tree_sitter::Node<'a>> {
    let mut iter = nodes.iter().copied();
    let mut root = iter.next()?;
    for node in iter {
        root = lca(root, node)?;
    }
    Some(root)
}

fn lca<'a>(
    mut left: tree_sitter::Node<'a>,
    mut right: tree_sitter::Node<'a>,
) -> Option<tree_sitter::Node<'a>> {
    if node_contains(left, right) {
        return Some(left);
    }
    if node_contains(right, left) {
        return Some(right);
    }
    while left.parent().is_some() || right.parent().is_some() {
        if left.start_byte() > right.start_byte() {
            left = left.parent()?;
        } else if right.start_byte() > left.start_byte() {
            right = right.parent()?;
        } else {
            let lp = left.parent()?;
            let rp = right.parent()?;
            if lp.id() == rp.id() {
                return Some(lp);
            }
            left = lp;
            right = rp;
        }
    }
    None
}
