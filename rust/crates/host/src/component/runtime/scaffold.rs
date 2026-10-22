//! Component scaffold.apply fan-out.

//! Per-instance component operations and scaffold.

use std::rc::Rc;

use codemod_recipe_lua::{eval_return_table, new_sandbox_with_require};
use mlua::{Function, Table};

use crate::component::entry;
use crate::component::runtime::context::{
    component_chunk_name, make_host, set_host_graph_snapshot, ComponentRunContext,
    ComponentScaffoldRequest,
};
use crate::component::runtime::finalize::finalize_via_facade;
use crate::component::scan::{
    component_incremental_scan, component_scan, ComponentIncrementalScanRequest,
};
use crate::component::types::ComponentInvokeResponse;
use crate::lua_runtime::{args_table_from_map, build_ctx};
use crate::recipe::registry::RecipeRegistry;


pub fn invoke_scaffold(
    registry: &mut RecipeRegistry,
    request: ComponentScaffoldRequest,
) -> ComponentInvokeResponse {
    if registry.stack().is_none() {
        return ComponentInvokeResponse::fail("stack manifest is required for component scaffold");
    }
    let Some(component) = registry.get_component(&request.component_id).cloned() else {
        return ComponentInvokeResponse::fail(format!(
            "component not found: {}",
            request.component_id
        ));
    };
    if !component.has_scaffold {
        return ComponentInvokeResponse::fail(format!(
            "component {} has no scaffold entry",
            request.component_id
        ));
    }

    // Ensure cache exists so post-scaffold detectChanges can glob-diff new roots,
    // and so scaffold.apply receives the pre-scaffold graph as 3rd arg.
    let scan = component_scan(registry);
    let scan_instances = scan.instances.unwrap_or_default();
    let scan_edges = scan.edges.unwrap_or_default();

    let ctx = ComponentRunContext::from_registry(registry, request.kind);
    let host = match make_host(&ctx, request.args.clone()) {
        Ok(h) => h,
        Err(message) => return ComponentInvokeResponse::fail(message),
    };
    if let Err(message) = set_host_graph_snapshot(&host, &scan_instances, &scan_edges) {
        return ComponentInvokeResponse::fail(message);
    }
    let lua = match new_sandbox_with_require(&ctx.codemod_root) {
        Ok(l) => l,
        Err(e) => return ComponentInvokeResponse::fail(e.to_string()),
    };
    let result = (|| -> Result<ComponentInvokeResponse, String> {
        let ctx_table = build_ctx(&lua, Rc::clone(&host))?;
        let name = component_chunk_name(&component);
        let module =
            eval_return_table(&lua, &component.source, &name).map_err(|e| e.to_string())?;
        let scaffold: Table = module
            .get("scaffold")
            .map_err(|e| format!("scaffold table missing: {e}"))?;
        let apply: Function = scaffold
            .get("apply")
            .map_err(|e| format!("scaffold apply missing: {e}"))?;
        let args_tbl = args_table_from_map(&lua, &request.args)?;
        // scaffold.apply(c, args, graph): the graph is the pre-scaffold snapshot —
        // same semantics as operations (judges the request, not mid-apply edits).
        let flat = crate::component::relations::flatten_instances(&scan_instances);
        let graph = crate::component::relations::build_graph_facade(&lua, &flat, &scan_edges)?;
        apply
            .call::<()>((ctx_table, args_tbl, graph))
            .map_err(|e| format!("scaffold apply failed: {e}"))?;

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

