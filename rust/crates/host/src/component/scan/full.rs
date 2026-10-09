//! Full component scan entry points.

use crate::component::cache::ComponentScanCache;
use crate::component::runtime::ComponentRunContext;
use crate::component::scan::graph_phases::run_graph_phases;
use crate::component::scan::identify::scan_component;
use crate::component::scan::types::ScanData;
use crate::component::types::ComponentScanResponse;
use crate::lua_runtime::RunKind;
use crate::recipe::registry::RecipeRegistry;

pub fn component_scan(registry: &mut RecipeRegistry) -> ComponentScanResponse {
    let response = component_scan_inner(registry);
    if response.ok && registry.persist_scan_snapshots() {
        // Best-effort architecture snapshot under codemod root (content-stable).
        let _ = crate::component::snapshot::persist_scan_snapshot(registry, &response, None);
    }
    response
}

/// Full scan + cache update without writing `{codemod_root}/.cache/scan-snapshot.json`.
pub fn component_scan_inner(registry: &mut RecipeRegistry) -> ComponentScanResponse {
    match run_full_scan(registry) {
        Ok(data) => {
            let response = scan_response(registry, &data);
            *registry.component_scan_cache_mut() =
                Some(ComponentScanCache::from_scan(registry, data));
            response
        }
        Err(message) => scan_error_with_diagnostics(registry, &message),
    }
}

pub(super) fn scan_error_with_diagnostics(registry: &RecipeRegistry, message: &str) -> ComponentScanResponse {
    ComponentScanResponse {
        diagnostics: if registry.diagnostics().is_empty() {
            None
        } else {
            Some(registry.diagnostics().to_vec())
        },
        ..scan_error(message)
    }
}

pub(super) fn scan_response(registry: &RecipeRegistry, data: &ScanData) -> ComponentScanResponse {
    ComponentScanResponse {
        ok: true,
        error: None,
        instances: Some(data.instances.clone()),
        findings: Some(data.all_findings()),
        edges: Some(data.edges.clone()),
        kind_graph: Some(data.kind_graph.clone()),
        diagnostics: if registry.diagnostics().is_empty() {
            None
        } else {
            Some(registry.diagnostics().to_vec())
        },
    }
}

pub(super) fn cached_response(registry: &RecipeRegistry, cache: &ComponentScanCache) -> ComponentScanResponse {
    ComponentScanResponse {
        ok: true,
        error: None,
        instances: Some(cache.instances.clone()),
        findings: Some(cache.all_findings()),
        edges: Some(cache.edges.clone()),
        kind_graph: Some(cache.kind_graph.clone()),
        diagnostics: if registry.diagnostics().is_empty() {
            None
        } else {
            Some(registry.diagnostics().to_vec())
        },
    }
}

pub(super) fn scan_error(message: &str) -> ComponentScanResponse {
    ComponentScanResponse {
        ok: false,
        error: Some(message.to_string()),
        instances: None,
        findings: None,
        edges: None,
        kind_graph: None,
        diagnostics: None,
    }
}

pub(super) fn run_full_scan(registry: &RecipeRegistry) -> Result<ScanData, String> {
    if registry.components().is_empty() {
        return Ok(ScanData::default());
    }
    let Some(stack) = registry.stack() else {
        return Err("stack manifest is required when components are loaded".to_string());
    };
    let ctx = ComponentRunContext::from_registry(registry, RunKind::Preview);
    let mut data = ScanData::default();
    for component_id in &stack.components {
        let Some(component) = registry.get_component(component_id) else {
            continue;
        };
        let (mut found, mut component_findings) = scan_component(&ctx, component)?;
        data.instances.append(&mut found);
        data.parse_findings.append(&mut component_findings);
    }
    run_graph_phases(&ctx, &mut data)?;
    Ok(data)
}
