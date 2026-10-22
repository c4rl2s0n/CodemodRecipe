//! Incremental re-scan of changed roots.

use std::collections::BTreeSet;

use crate::component::runtime::ComponentRunContext;
use crate::component::scan::full::{cached_response, component_scan, scan_error};
use crate::component::scan::graph_phases::run_graph_phases;
use crate::component::scan::identify::{identify_diff, rescan_root_context};
use crate::component::scan::types::{ComponentIncrementalScanRequest, ScanData};
use crate::component::types::ComponentScanResponse;
use crate::lua_runtime::RunKind;
use crate::recipe::registry::RecipeRegistry;

pub fn component_incremental_scan(
    registry: &mut RecipeRegistry,
    request: ComponentIncrementalScanRequest,
) -> ComponentScanResponse {
    if request.force || registry.component_scan_cache().is_none() {
        return component_scan(registry);
    }

    let cache = registry.component_scan_cache().cloned().unwrap();

    if !request.detect_changes && request.dirty_paths.is_empty() {
        return cached_response(registry, &cache);
    }

    let mut new_roots = Vec::new();
    let mut removed_roots = Vec::new();
    let mut roots_to_reparse = BTreeSet::new();

    if request.detect_changes {
        let mut detect = cache.detect_dirty_paths(registry, &request.dirty_paths);
        if let Err(message) = identify_diff(registry, &cache, &mut detect) {
            return scan_error(&message);
        }
        new_roots = detect.new_roots;
        removed_roots = detect.removed_roots;
        roots_to_reparse.extend(detect.roots_to_reparse);
    } else {
        roots_to_reparse.extend(cache.roots_for_dirty_paths(&request.dirty_paths));
    }

    if roots_to_reparse.is_empty() && new_roots.is_empty() && removed_roots.is_empty() {
        return cached_response(registry, &cache);
    }

    let ctx = ComponentRunContext::from_registry(registry, RunKind::Preview);
    let mut updated_instances = Vec::new();
    let mut updated_parse_findings = Vec::new();

    for instance_id in &roots_to_reparse {
        let Some(entry) = cache.entry_for_instance(instance_id) else {
            continue;
        };
        let Some(component) = registry.get_component(&entry.component_id) else {
            continue;
        };
        match rescan_root_context(&ctx, component, entry.locator.clone()) {
            Ok((mut instances, mut findings)) => {
                updated_instances.append(&mut instances);
                updated_parse_findings.append(&mut findings);
            }
            Err(message) => return scan_error(&message),
        }
    }

    for root in &new_roots {
        let Some(component) = registry.get_component(&root.component_id) else {
            continue;
        };
        match rescan_root_context(&ctx, component, root.locator.clone()) {
            Ok((mut instances, mut findings)) => {
                updated_instances.append(&mut instances);
                updated_parse_findings.append(&mut findings);
            }
            Err(message) => return scan_error(&message),
        }
    }

    let mut cache = cache;
    cache.merge_rescan(
        registry,
        &removed_roots,
        updated_instances,
        updated_parse_findings,
    );

    // Relations + judgment always re-run over the merged forest: a dirty file can
    // change edges between otherwise unchanged instances.
    let mut data = ScanData {
        instances: cache.instances.clone(),
        parse_findings: cache.parse_findings.clone(),
        ..Default::default()
    };
    if let Err(message) = run_graph_phases(&ctx, &mut data) {
        return scan_error(&message);
    }
    cache.edges = data.edges.clone();
    cache.kind_graph = data.kind_graph.clone();
    cache.judgment_findings = data.judgment_findings.clone();
    cache.instances = data.instances.clone();

    let response = cached_response(registry, &cache);
    *registry.component_scan_cache_mut() = Some(cache);
    response
}
