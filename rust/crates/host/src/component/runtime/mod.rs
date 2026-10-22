//! Component/stack invoke runtime: context, hooks, operations, finalize.

mod context;
mod finalize;
mod hook;
mod operations;
mod scaffold;
mod stack_ops;

pub use context::{ComponentInvokeRequest, ComponentRunContext, ComponentScaffoldRequest};
pub(crate) use context::{
    invoke_component_hook, invoke_component_hook_with_facade,
    make_host, try_invoke_component_hook,
};
pub use finalize::hook_to_json;
pub use hook::{invoke_hook, ComponentHookRequest};
pub use operations::invoke_operation;
pub use scaffold::invoke_scaffold;
pub use stack_ops::{invoke_stack_operation, StackInvokeRequest};
