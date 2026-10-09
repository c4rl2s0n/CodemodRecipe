pub mod specs;

pub use specs::{
    Anchor, ByteSpan, CreateSpec, DeleteSpec, Env, HostSpec, HostType, IfExists, IfMissing,
    InsertSpec, MatchFileRow, PostExecSpec, QueryCaptureSpec, QueryHit, QueryMatchRow, QuerySpec,
    QueryTableSpec, ReadSpec, RemoveSpec, ReplaceSpec, SpanTextSpec, TemplateSpec,
};
