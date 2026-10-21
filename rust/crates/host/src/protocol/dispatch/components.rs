//! Component / stack HostCommand handlers.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::component::scan::component_scan_inner;
use crate::component::{
    component_coverage, component_incremental_scan, component_scan, invoke_hook, invoke_operation,
    invoke_scaffold, invoke_stack_operation, ComponentHookRequest, ComponentIncrementalScanRequest,
    ComponentInvokeRequest, ComponentScaffoldRequest, StackInvokeRequest,
};
use crate::component::types::ComponentInvokeResponse;
use crate::lua_runtime::RunKind;
use crate::preview::preview_token::{compute_preview_token, validate_preview_token};
use crate::protocol::dispatch::common::{to_value, write_workspace_relative};
use crate::protocol::{ComponentGraphResponse, DescribeComponentResponse, FilePreview};
use crate::recipe::registry::RecipeRegistry;
use crate::sandbox::path_sandbox::PathSandbox;

pub(super) fn component_snapshot(
    registry: &mut RecipeRegistry,
    relative: Option<String>,
) -> serde_json::Value {
    let scan = component_scan_inner(registry);
    if !scan.ok {
        return serde_json::json!({
            "ok": false,
            "error": scan.error.unwrap_or_else(|| "component scan failed".to_string()),
        });
    }
    let path_arg = relative.as_deref().map(str::trim).filter(|s| !s.is_empty());
    match crate::component::snapshot::persist_scan_snapshot(registry, &scan, path_arg) {
        Ok(result) => serde_json::json!({
            "ok": true,
            "path": result.path,
            "wrote": result.wrote,
            "kindGraphPath": result.kind_graph_path,
            "instanceGraphPath": result.instance_graph_path,
            "graphsWrote": result.graphs_wrote,
            "meta": {
                "instanceCount": result.instance_count,
                "edgeCount": result.edge_count,
                "findingCount": result.finding_count,
            },
        }),
        Err(message) => serde_json::json!({ "ok": false, "error": message }),
    }
}

pub(super) fn scan(registry: &mut RecipeRegistry) -> serde_json::Value {
    to_value(component_scan(registry))
}

pub(super) fn incremental_scan(
    registry: &mut RecipeRegistry,
    dirty_paths: Vec<String>,
    detect_changes: bool,
    force: bool,
) -> serde_json::Value {
    to_value(component_incremental_scan(
        registry,
        ComponentIncrementalScanRequest {
            dirty_paths,
            detect_changes,
            force,
        },
    ))
}

pub(super) fn list_instances(registry: &mut RecipeRegistry) -> serde_json::Value {
    let scan = component_scan(registry);
    to_value(serde_json::json!({
        "ok": scan.ok,
        "error": scan.error,
        "summary": scan.summary,
        "instances": scan.instances,
        "findings": scan.findings,
        "diagnostics": scan.diagnostics,
    }))
}

pub(super) fn coverage(registry: &mut RecipeRegistry) -> serde_json::Value {
    to_value(component_coverage(registry))
}

pub(super) fn graph(
    registry: &mut RecipeRegistry,
    level: Option<String>,
    out: Option<String>,
) -> serde_json::Value {
    let level = level.as_deref().unwrap_or("kinds");
    if level != "kinds" && level != "instances" {
        return to_value(ComponentGraphResponse {
            ok: false,
            error: Some(format!(
                "unknown componentGraph level '{level}' (expected 'kinds' or 'instances')"
            )),
            mermaid: None,
            kind_graph: None,
            edges: None,
            path: None,
        });
    }
    let scan = component_scan(registry);
    if !scan.ok {
        return to_value(ComponentGraphResponse {
            ok: false,
            error: Some(
                scan.error
                    .unwrap_or_else(|| "component scan failed".to_string()),
            ),
            mermaid: None,
            kind_graph: None,
            edges: None,
            path: None,
        });
    }
    let kind_graph = scan.kind_graph.clone().unwrap_or_default();
    let edges = scan.edges.clone().unwrap_or_default();
    let (mermaid, out_edges) = if level == "instances" {
        let instances = scan.instances.unwrap_or_default();
        let flat = crate::component::relations::flatten_instances(&instances);
        let member_kinds = crate::component::member_kinds(
            &registry
                .stack()
                .map(|s| s.relations.clone())
                .unwrap_or_default(),
        );
        (
            crate::component::render_instance_graph_mermaid(&edges, &flat, &member_kinds),
            Some(edges),
        )
    } else {
        (crate::component::render_kind_graph_mermaid(&kind_graph), None)
    };
    let written = match write_workspace_relative(registry, out.as_deref(), mermaid.as_bytes()) {
        Ok(p) => p,
        Err(message) => {
            return to_value(ComponentGraphResponse {
                ok: false,
                error: Some(message),
                mermaid: Some(mermaid),
                kind_graph: Some(kind_graph),
                edges: out_edges,
                path: None,
            });
        }
    };
    to_value(ComponentGraphResponse {
        ok: true,
        error: None,
        mermaid: Some(mermaid),
        kind_graph: Some(kind_graph),
        edges: out_edges,
        path: written,
    })
}

pub(super) fn invoke_operation_cmd(
    registry: &mut RecipeRegistry,
    instance_id: String,
    operation: String,
    args: std::collections::BTreeMap<String, serde_json::Value>,
    preview: bool,
) -> serde_json::Value {
    to_value(invoke_operation(
        registry,
        ComponentInvokeRequest {
            instance_id,
            operation,
            args,
            kind: if preview {
                RunKind::Preview
            } else {
                RunKind::Apply
            },
        },
    ))
}

pub(super) fn scaffold(
    registry: &mut RecipeRegistry,
    component_id: String,
    args: std::collections::BTreeMap<String, serde_json::Value>,
    preview: bool,
) -> serde_json::Value {
    to_value(invoke_scaffold(
        registry,
        ComponentScaffoldRequest {
            component_id,
            args,
            kind: if preview {
                RunKind::Preview
            } else {
                RunKind::Apply
            },
        },
    ))
}

pub(super) fn invoke_hook_cmd(
    registry: &mut RecipeRegistry,
    component_id: String,
    hook: String,
    preview: bool,
    locator: Option<serde_json::Value>,
    instance: Option<serde_json::Value>,
    operation: Option<String>,
    args: BTreeMap<String, serde_json::Value>,
    preview_token: Option<String>,
) -> serde_json::Value {
    let identity = hook_identity(&component_id, &hook, &locator, &instance, &operation);
    let make_req = |kind: RunKind| ComponentHookRequest {
        component_id: component_id.clone(),
        hook: hook.clone(),
        kind,
        locator: locator.clone(),
        instance: instance.clone(),
        operation: operation.clone(),
        args: args.clone(),
    };

    if !preview {
        let dry = invoke_hook(registry, make_req(RunKind::Preview));
        if !dry.ok {
            return to_value(dry);
        }
        if let Err(message) = validate_hook_token(registry, &identity, &args, &dry, &preview_token)
        {
            return to_value(ComponentInvokeResponse::fail(message));
        }
    }

    let mut resp = invoke_hook(
        registry,
        make_req(if preview {
            RunKind::Preview
        } else {
            RunKind::Apply
        }),
    );
    if preview && resp.ok {
        resp.preview_token = Some(token_for_response(registry, &identity, &args, &resp));
    }
    to_value(resp)
}

pub(super) fn stack_invoke(
    registry: &mut RecipeRegistry,
    operation: String,
    preview: bool,
    args: BTreeMap<String, serde_json::Value>,
    preview_token: Option<String>,
) -> serde_json::Value {
    let identity = serde_json::json!({
        "kind": "stackOperation",
        "operation": operation,
    });
    let make_req = |kind: RunKind| StackInvokeRequest {
        operation: operation.clone(),
        args: args.clone(),
        kind,
    };

    if !preview {
        let dry = invoke_stack_operation(registry, make_req(RunKind::Preview));
        if !dry.ok {
            return to_value(dry);
        }
        if let Err(message) = validate_hook_token(registry, &identity, &args, &dry, &preview_token)
        {
            return to_value(ComponentInvokeResponse::fail(message));
        }
    }

    let mut resp = invoke_stack_operation(
        registry,
        make_req(if preview {
            RunKind::Preview
        } else {
            RunKind::Apply
        }),
    );
    if preview && resp.ok {
        resp.preview_token = Some(token_for_response(registry, &identity, &args, &resp));
    }
    to_value(resp)
}

pub(super) fn describe_component(
    registry: &RecipeRegistry,
    component_id: String,
) -> serde_json::Value {
    match registry.describe_component(&component_id) {
        Some(component) => to_value(DescribeComponentResponse {
            ok: true,
            error: None,
            component: Some(component),
        }),
        None => to_value(DescribeComponentResponse {
            ok: false,
            error: Some(format!("component not found: {component_id}")),
            component: None,
        }),
    }
}

fn hook_identity(
    component_id: &str,
    hook: &str,
    locator: &Option<serde_json::Value>,
    instance: &Option<serde_json::Value>,
    operation: &Option<String>,
) -> serde_json::Value {
    serde_json::json!({
        "kind": "componentHook",
        "componentId": component_id,
        "hook": hook,
        "locator": locator,
        "instance": instance,
        "operation": operation,
    })
}

fn validate_hook_token(
    registry: &RecipeRegistry,
    identity: &serde_json::Value,
    args: &BTreeMap<String, serde_json::Value>,
    dry: &ComponentInvokeResponse,
    preview_token: &Option<String>,
) -> Result<(), String> {
    let provided = preview_token.as_deref().unwrap_or("");
    let paths = snapshot_paths_from_files(registry, dry.files.as_deref());
    let path_refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
    validate_preview_token(None, Some(identity), args, provided, &path_refs)
}

fn token_for_response(
    registry: &RecipeRegistry,
    identity: &serde_json::Value,
    args: &BTreeMap<String, serde_json::Value>,
    resp: &ComponentInvokeResponse,
) -> String {
    let paths = snapshot_paths_from_files(registry, resp.files.as_deref());
    let path_refs: Vec<&Path> = paths.iter().map(|p| p.as_path()).collect();
    compute_preview_token(None, Some(identity), args, &path_refs)
}

fn snapshot_paths_from_files(
    registry: &RecipeRegistry,
    files: Option<&[FilePreview]>,
) -> Vec<PathBuf> {
    let Some(files) = files else {
        return Vec::new();
    };
    let sandbox = PathSandbox::new(registry.workspace_root.clone());
    files
        .iter()
        .filter(|f| !f.skipped)
        .filter_map(|f| sandbox.resolve_workspace_relative(&f.path).ok())
        .collect()
}
