//! Per-instance component operations and scaffold.

use std::collections::BTreeMap;
use std::rc::Rc;

use codemod_recipe_lua::json_lua::{json_to_lua, lua_to_json};
use codemod_recipe_lua::{eval_return_table, new_sandbox_with_require, require_chunk_name};
use mlua::{Function, Lua, Table, Value};

use crate::component::relations::{build_graph_facade, flatten_instances};
use crate::component::runtime::context::{
    component_chunk_name, facade_node, make_host, set_host_graph_snapshot, ComponentInvokeRequest,
    ComponentRunContext, ComponentScaffoldRequest,
};
use crate::component::runtime::finalize::{component_scan_after_operation, finalize_host};
use crate::component::scan::{
    component_incremental_scan, component_scan, ComponentIncrementalScanRequest,
};
use crate::component::types::{ComponentFinding, ComponentInstance, ComponentInvokeResponse};
use crate::engine::working_tree::WorkingTree;
use crate::host_api::context::SharedHost;
use crate::lua_runtime::{args_table_from_map, build_ctx, RunKind};
use crate::preview::diff::build_file_preview_from_change;
use crate::recipe::registry::RecipeRegistry;

pub fn invoke_operation(
    registry: &mut RecipeRegistry,
    request: ComponentInvokeRequest,
) -> ComponentInvokeResponse {
    if registry.stack().is_none() {
        return ComponentInvokeResponse::fail(
            "stack manifest is required for component operations",
        );
    }
    let ctx = ComponentRunContext::from_registry(registry, request.kind);
    let scan = crate::component::scan::component_scan(registry);
    if !scan.ok {
        return ComponentInvokeResponse::fail(
            scan.error
                .unwrap_or_else(|| "component scan failed".to_string()),
        );
    }
    let instances = scan.instances.unwrap_or_default();
    let edges = scan.edges.clone().unwrap_or_default();
    let Some(target) = instances.iter().find(|i| i.id == request.instance_id) else {
        return ComponentInvokeResponse::fail(format!(
            "instance not found: {}",
            request.instance_id
        ));
    };
    let component_id = target.component_id.clone();
    let Some(component) = registry.get_component(&component_id).cloned() else {
        return ComponentInvokeResponse::fail(format!("component not found: {component_id}"));
    };
    if !component
        .operations
        .iter()
        .any(|op| op.name == request.operation)
    {
        return ComponentInvokeResponse::fail(format!(
            "unknown operation {} on component {}",
            request.operation, component_id
        ));
    }
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
        let name = component_chunk_name(&component);
        let module =
            eval_return_table(&lua, &component.source, &name).map_err(|e| e.to_string())?;
        let ops: Table = module
            .get("operations")
            .map_err(|e| format!("operations table missing: {e}"))?;
        let op_tbl: Table = ops
            .get(request.operation.as_str())
            .map_err(|_| format!("operation {} not found", request.operation))?;
        let apply: Function = op_tbl
            .get("apply")
            .map_err(|e| format!("operation apply missing: {e}"))?;
        let args_tbl = args_table_from_map(&lua, &request.args)?;
        // Operations receive the pre-operation graph as 4th arg (guards, navigation).
        let flat = crate::component::relations::flatten_instances(&instances);
        let graph = crate::component::relations::build_graph_facade(&lua, &flat, &edges)?;
        let instance_json = serde_json::json!({
            "id": target.id,
            "componentId": target.component_id,
            "locator": target.locator,
            "ast": target.ast,
        });
        // Pass the facade's own node table so `target:outgoing(...)` etc. work.
        let instance_tbl = match facade_node(&graph, &instance_json) {
            Some(table) => Value::Table(table),
            None => json_to_lua(&lua, &instance_json)?,
        };
        apply
            .call::<()>((ctx_table, args_tbl, instance_tbl, graph))
            .map_err(|e| format!("operation apply failed: {e}"))?;

        let mut inner = host.lock().map_err(|e| e.to_string())?;
        let applied = if request.kind == RunKind::Apply {
            Some(inner.flush_to_disk()?)
        } else {
            None
        };
        let changes = std::mem::replace(&mut inner.tree, WorkingTree::new()).finalize();
        let transcript = std::mem::take(&mut inner.transcript);
        drop(inner);

        let rescanned = component_scan_after_operation(registry, &target.id, &changes)?;
        let files = if changes.is_empty() {
            None
        } else {
            Some(
                changes
                    .iter()
                    .filter_map(|c| build_file_preview_from_change(c, false, None).ok())
                    .collect(),
            )
        };
        Ok(ComponentInvokeResponse {
            ok: true,
            error: None,
            result: None,
            instance: rescanned.instance,
            findings: rescanned.findings,
            files,
            transcript: None,
            logs: None,
            applied,
        }
        .with_transcript(Some(transcript)))
    })();
    match result {
        Ok(resp) => resp,
        Err(message) => {
            let transcript = host
                .lock()
                .ok()
                .map(|mut inner| std::mem::take(&mut inner.transcript))
                .filter(|t| !t.is_empty());
            ComponentInvokeResponse::fail(message).with_transcript(transcript)
        }
    }
}

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

        let mut inner = host.lock().map_err(|e| e.to_string())?;
        let applied = if request.kind == RunKind::Apply {
            Some(inner.flush_to_disk()?)
        } else {
            None
        };
        let changes = std::mem::replace(&mut inner.tree, WorkingTree::new()).finalize();
        let transcript = std::mem::take(&mut inner.transcript);
        drop(inner);

        let scan = component_incremental_scan(
            registry,
            ComponentIncrementalScanRequest {
                dirty_paths: changes.iter().map(|c| c.path().to_string()).collect(),
                detect_changes: true,
                ..Default::default()
            },
        );
        if !scan.ok {
            return Err(scan.error.unwrap_or_else(|| "rescan failed".to_string()));
        }
        let files = if changes.is_empty() {
            None
        } else {
            Some(
                changes
                    .iter()
                    .filter_map(|c| build_file_preview_from_change(c, false, None).ok())
                    .collect(),
            )
        };
        Ok(ComponentInvokeResponse {
            ok: true,
            error: None,
            result: None,
            instance: None,
            findings: scan.findings,
            files,
            transcript: None,
            logs: None,
            applied,
        }
        .with_transcript(Some(transcript)))
    })();
    match result {
        Ok(resp) => resp,
        Err(message) => {
            let transcript = host
                .lock()
                .ok()
                .map(|mut inner| std::mem::take(&mut inner.transcript))
                .filter(|t| !t.is_empty());
            ComponentInvokeResponse::fail(message).with_transcript(transcript)
        }
    }
}

