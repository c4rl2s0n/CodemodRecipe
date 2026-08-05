use serde::Serialize;
use tree_sitter::{Node, Tree};

use crate::QueryToolsError;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct GeneratedQuery {
    pub query: String,
    pub capture_suggestion: String,
}

#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub include_text_predicates: bool,
    pub capture_leaf: String,
    /// Max ancestors to include above the leaf (including leaf). None = all to root.
    pub max_depth: Option<usize>,
}

impl Default for GenerateOptions {
    fn default() -> Self {
        Self {
            include_text_predicates: false,
            capture_leaf: "target".to_string(),
            max_depth: Some(8),
        }
    }
}

/// Generate a starter tree-sitter query for the named node covering `byte_offset`
/// (or the node spanning `[start, end)` if provided).
pub fn generate_query(
    tree: &Tree,
    source: &str,
    start: usize,
    end: usize,
    opts: &GenerateOptions,
) -> Result<GeneratedQuery, QueryToolsError> {
    let root = tree.root_node();
    let point = if end > start {
        start + (end - start) / 2
    } else {
        start
    };
    let mut node = root
        .named_descendant_for_byte_range(point, point.max(point + 1).min(source.len().max(1)))
        .or_else(|| root.descendant_for_byte_range(point, point.max(point + 1)))
        .ok_or(QueryToolsError::NoNodeAtOffset(point))?;

    // Prefer named node
    while !node.is_named() {
        node = node
            .parent()
            .ok_or(QueryToolsError::NoNodeAtOffset(point))?;
    }

    // If range was provided, try to find a node that better matches the span
    if end > start {
        if let Some(n) = root.named_descendant_for_byte_range(start, end) {
            if n.is_named() {
                node = n;
            }
        }
    }

    let mut chain: Vec<Node<'_>> = Vec::new();
    let mut cur = Some(node);
    while let Some(n) = cur {
        if n.id() == root.id() {
            break;
        }
        if n.is_named() {
            chain.push(n);
        }
        cur = n.parent();
    }
    chain.reverse();

    if let Some(max) = opts.max_depth {
        if chain.len() > max {
            let skip = chain.len() - max;
            chain = chain[skip..].to_vec();
        }
    }

    if chain.is_empty() {
        return Err(QueryToolsError::NoNodeAtOffset(point));
    }

    let capture = opts.capture_leaf.clone();
    let mut predicates = Vec::new();
    let query = emit_chain(&chain, source, &capture, opts, &mut predicates, 0);

    let mut full = query;
    for p in predicates {
        full.push('\n');
        full.push_str(&p);
    }

    Ok(GeneratedQuery {
        query: full,
        capture_suggestion: capture,
    })
}

fn emit_chain(
    chain: &[Node<'_>],
    source: &str,
    capture: &str,
    opts: &GenerateOptions,
    predicates: &mut Vec<String>,
    index: usize,
) -> String {
    let node = chain[index];
    let kind = node.kind();
    let is_leaf = index + 1 >= chain.len();
    let indent = "  ".repeat(index);

    // Field name from parent
    let field = if index > 0 {
        field_name_of_child(chain[index - 1], node)
    } else {
        None
    };

    let mut s = String::new();
    if let Some(f) = &field {
        s.push_str(&indent);
        s.push_str(f);
        s.push_str(": ");
    } else {
        s.push_str(&indent);
    }

    if is_leaf {
        s.push('(');
        s.push_str(kind);
        s.push_str(") @");
        s.push_str(capture);
        if is_last_named_child(node) {
            s.push_str(" .");
        }
        if opts.include_text_predicates && (kind == "identifier" || kind.contains("string")) {
            if let Some(text) = source.get(node.start_byte()..node.end_byte()) {
                let lit = text.trim_matches(|c| c == '"' || c == '\'');
                if !lit.is_empty() && lit.len() < 64 && !lit.contains('\n') {
                    predicates.push(format!(
                        "(#eq? @{} \"{}\")",
                        capture,
                        escape_str(lit)
                    ));
                }
            }
        }
        return s;
    }

    s.push('(');
    s.push_str(kind);
    s.push('\n');
    s.push_str(&emit_chain(
        chain,
        source,
        capture,
        opts,
        predicates,
        index + 1,
    ));
    s.push('\n');
    s.push_str(&indent);
    s.push(')');
    s
}

fn field_name_of_child(parent: Node<'_>, child: Node<'_>) -> Option<String> {
    let count = parent.child_count();
    for i in 0..count {
        if let Some(c) = parent.child(i as u32) {
            if c.id() == child.id() {
                return parent.field_name_for_child(i as u32).map(|s| s.to_string());
            }
        }
    }
    None
}

fn is_last_named_child(node: Node<'_>) -> bool {
    let parent = match node.parent() {
        Some(p) => p,
        None => return false,
    };
    let count = parent.named_child_count();
    if count == 0 {
        return false;
    }
    parent
        .named_child((count - 1) as u32)
        .map(|last| last.id() == node.id())
        .unwrap_or(false)
}

fn escape_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
