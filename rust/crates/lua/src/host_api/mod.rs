pub mod specs;

pub use specs::{
    Anchor, ApplyTemplateSpec, CreateSpec, DeleteSpec, Env, HostSpec, IfExists, IfMissing,
    InsertSpec, PostExecSpec, QuerySpec, QueryTableSpec, ReadSpec, RemoveSpec,
    RenderFileTemplateSpec, ReplaceSpec,
};
