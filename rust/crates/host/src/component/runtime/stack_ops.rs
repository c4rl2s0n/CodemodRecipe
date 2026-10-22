//! Stack-level operation invoke.

use std::collections::BTreeMap;
use std::rc::Rc;

use codemod_recipe_lua::{eval_return_table, new_sandbox_with_require, require_chunk_name};
use mlua::{Function, Table};

use crate::component::entry;
use crate::component::runtime::context::{
    make_host, set_host_graph_snapshot, ComponentRunContext,
};
use crate::component::runtime::finalize::finalize_via_facade;
use crate::component::scan::{
    component_incremental_scan, component_scan, ComponentIncrementalScanRequest,
};
use crate::component::types::ComponentInvokeResponse;
use crate::lua_runtime::{args_table_from_map, build_ctx, RunKind};
use crate::recipe::registry::RecipeRegistry;

pub struct StackInvokeRequest {
    pub operation: String,
    pub args: BTreeMap<String, serde_json::Value>,
    pub kind: RunKind,
}

/// Dry-run / apply a stack-level operation (`stack.luau` `operations.<name>.apply`).
/// Instance-less (like scaffold); receives the pre-operation graph as 3rd arg.
pub fn invoke_stack_operation(
    registry: &mut RecipeRegistry,
    request: StackInvokeRequest,
) -> ComponentInvokeResponse {
    let Some(stack) = registry.stack().cloned() else {
        return ComponentInvokeResponse::fail("stack manifest is required for stack operations");
    };
    if !stack
        .operations
        .iter()
        .any(|op| op.name == request.operation)
    {
        return ComponentInvokeResponse::fail(format!(
            "unknown stack operation '{}'",
            request.operation
        ));
    }

    let scan = component_scan(registry);
    if !scan.ok {
        return ComponentInvokeResponse::fail(
            scan.error
                .unwrap_or_else(|| "component scan failed".to_string()),
        );
    }
    let instances = scan.instances.unwrap_or_default();
    let edges = scan.edges.unwrap_or_default();

    let ctx = ComponentRunContext::from_registry(registry, request.kind);
    let host = match make_host(&ctx, request.args.clone()) {
        Ok(h) => h,
        Err(message) => return ComponentInvokeResponse::fail(message),
    };
    if let Err(message) = set_host_graph_snapshot(&host, &instances, &edges) {
        return ComponentInvokeResponse::fail(message);
    }
    let lua = match new_sandbox_with_require(&ctx.codemod_root) {
        Ok(l) => l,
        Err(e) => return ComponentInvokeResponse::fail(e.to_string()),
    };
    let result = (|| -> Result<ComponentInvokeResponse, String> {
        let ctx_table = build_ctx(&lua, Rc::clone(&host))?;
        let name = stack
            .path
            .as_ref()
            .map(|p| require_chunk_name(p))
            .unwrap_or_else(|| stack.id.clone());
        let module = eval_return_table(&lua, &stack.source, &name).map_err(|e| e.to_string())?;
        let ops: Table = module
            .get("operations")
            .map_err(|e| format!("stack operations table missing: {e}"))?;
        let op_tbl: Table = ops
            .get(request.operation.as_str())
            .map_err(|_| format!("stack operation {} not found", request.operation))?;
        let apply: Function = op_tbl
            .get("apply")
            .map_err(|e| format!("stack operation apply missing: {e}"))?;
        let args_tbl = args_table_from_map(&lua, &request.args)?;
        let flat = crate::component::relations::flatten_instances(&instances);
        let graph = crate::component::relations::build_graph_facade(&lua, &flat, &edges)?;
        apply
            .call::<()>((ctx_table, args_tbl, graph))
            .map_err(|e| format!("stack operation apply failed: {e}"))?;

        let parts =
            entry::with_component_host(&host, |h| finalize_via_facade(h, request.kind))?;
        let scan = component_incremental_scan(
            registry,
            ComponentIncrementalScanRequest {
                dirty_paths: parts.changes.iter().map(|c| c.path().to_string()).collect(),
                detect_changes: true,
                ..Default::default()
            },
        );
        if !scan.ok {
            return Err(scan.error.unwrap_or_else(|| "rescan failed".to_string()));
        }
        Ok(ComponentInvokeResponse {
            ok: true,
            error: None,
            result: None,
            instance: None,
            findings: scan.findings,
            files: parts.files,
            transcript: None,
            logs: None,
            applied: parts.applied,
            preview_token: None,
        }
        .with_transcript(Some(parts.transcript)))
    })();
    match result {
        Ok(resp) => resp,
        Err(message) => {
            let transcript = entry::with_component_host(&host, |h| Ok(h.take_transcript()))
                .ok()
                .filter(|t| !t.is_empty());
            ComponentInvokeResponse::fail(message).with_transcript(transcript)
        }
    }
}

