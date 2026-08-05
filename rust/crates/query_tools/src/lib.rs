//! Tree-sitter query authoring helpers: AST dump, match debug, and query generation.
//!
//! Pure `Language` + source APIs — no recipe YAML, Jinja, or workspace coupling.

mod dump;
mod generate;
mod instrument;
mod match_debug;
mod position;

pub use dump::{dump_ast, AstNode, DumpOptions, Position};
pub use generate::{generate_query, GenerateOptions, GeneratedQuery};
pub use instrument::{instrument_query, InstrumentedQuery};
pub use match_debug::{
    debug_query, parse_tree, CaptureInfo, DebugMatch, DebugOptions, DebugQueryResult, NodeSpan,
};
pub use position::{byte_to_position, positions_for_range};

use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum QueryToolsError {
    #[error("{0}")]
    Query(String),
    #[error("parse failed")]
    ParseFailed,
    #[error("no named node at byte offset {0}")]
    NoNodeAtOffset(usize),
}
