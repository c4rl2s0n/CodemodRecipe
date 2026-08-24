use serde::Serialize;
use tree_sitter::Node;

use crate::position::byte_to_position;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Position {
    pub byte: u32,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AstNode {
    pub kind: String,
    pub named: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    pub start: Position,
    pub end: Position,
    pub is_error: bool,
    pub is_missing: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    pub children: Vec<AstNode>,
}

#[derive(Debug, Clone)]
pub struct DumpOptions {
    /// When true (default), skip anonymous (punctuation) nodes.
    pub named_only: bool,
    /// Include leaf text when length <= max_text_len.
    pub include_text: bool,
    pub max_text_len: usize,
}

impl Default for DumpOptions {
    fn default() -> Self {
        Self {
            named_only: true,
            include_text: true,
            max_text_len: 80,
        }
    }
}

/// Dump the syntax tree rooted at `root` as a JSON-friendly structure.
pub fn dump_ast(root: Node<'_>, source: &str, opts: &DumpOptions) -> AstNode {
    dump_node(root, source, opts, None)
}

fn dump_node(node: Node<'_>, source: &str, opts: &DumpOptions, field: Option<String>) -> AstNode {
    let start = node.start_byte();
    let end = node.end_byte();
    let text = if opts.include_text && end.saturating_sub(start) <= opts.max_text_len {
        source.get(start..end).map(|s| s.to_string())
    } else {
        None
    };

    let mut children = Vec::new();
    let count = node.child_count();
    for i in 0..count {
        let Some(child) = node.child(i as u32) else {
            continue;
        };
        if opts.named_only && !child.is_named() {
            continue;
        }
        let field_name = node.field_name_for_child(i as u32).map(|s| s.to_string());
        children.push(dump_node(child, source, opts, field_name));
    }

    AstNode {
        kind: node.kind().to_string(),
        named: node.is_named(),
        field,
        start: byte_to_position(source, start),
        end: byte_to_position(source, end),
        is_error: node.is_error() || node.kind() == "ERROR",
        is_missing: node.is_missing(),
        text,
        children,
    }
}
