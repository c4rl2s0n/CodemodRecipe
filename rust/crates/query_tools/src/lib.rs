pub mod debug;
pub mod position;
pub mod query;

pub use debug::instrument::{instrument_query, InstrumentedQuery};
pub use debug::match_debug::{
    debug_query, parse_tree, CaptureInfo, DebugMatch, DebugOptions, DebugQueryResult, NodeSpan,
};
pub use position::{byte_to_position, positions_for_range};
pub use query::dump::{dump_ast, AstNode, DumpOptions, Position};
pub use query::generate::{generate_query, GenerateOptions, GeneratedQuery};

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
