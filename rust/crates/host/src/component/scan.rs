use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use codemod_recipe_lua::model::ComponentDef;

use crate::component::cache::{locator_key, ComponentScanCache, NewRoot, RootKey};
use crate::component::relations::{
    attach_relations, build_graph_facade, build_index_facade, flatten_instances, kind_graph,
    normalize_raw_edges, resolve_edges, ComponentEdge, FlatInstance, KindEdge, RawEdge,
};
use crate::component::runtime::{
    hook_to_json, invoke_component_hook, invoke_component_hook_with_facade, make_host,
    try_invoke_component_hook, ComponentRunContext,
};
use crate::component::types::{
    expand_parse_results, instance_id, normalize_findings, normalize_parse_context,
    soft_validate_parsed, ComponentFinding, ComponentInstance, ComponentScanResponse,
};
use crate::host_api::context::SharedHost;
use crate::lua_runtime::RunKind;
use crate::recipe::registry::RecipeRegistry;

#[derive(Debug, Clone, Default)]
pub struct ComponentIncrementalScanRequest {
    pub dirty_paths: Vec<String>,
    pub detect_changes: bool,
    pub force: bool,
}

/// Everything a scan produces, split by origin so incremental merges stay precise.
#[derive(Debug, Clone, Default)]
pub struct ScanData {
    pub instances: Vec<ComponentInstance>,
    /// Parse-time findings (soft validation) — merged per-instance on incremental scans.
    pub parse_findings: Vec<ComponentFinding>,
    /// Relation + verify findings — recomputed wholesale over the merged forest.
    pub judgment_findings: Vec<ComponentFinding>,
    pub edges: Vec<ComponentEdge>,
    pub kind_graph: Vec<KindEdge>,
}

impl ScanData {
    pub fn all_findings(&self) -> Vec<ComponentFinding> {
        let mut findings = self.parse_findings.clone();
        findings.extend(self.judgment_findings.clone());
        findings
    }
}

pub fn component_scan(registry: &mut RecipeRegistry) -> ComponentScanResponse {
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

fn scan_error_with_diagnostics(registry: &RecipeRegistry, message: &str) -> ComponentScanResponse {
    ComponentScanResponse {
        diagnostics: if registry.diagnostics().is_empty() {
            None
        } else {
            Some(registry.diagnostics().to_vec())
        },
        ..scan_error(message)
    }
}

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

fn scan_response(registry: &RecipeRegistry, data: &ScanData) -> ComponentScanResponse {
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

fn cached_response(registry: &RecipeRegistry, cache: &ComponentScanCache) -> ComponentScanResponse {
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

fn scan_error(message: &str) -> ComponentScanResponse {
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

fn run_full_scan(registry: &RecipeRegistry) -> Result<ScanData, String> {
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

/// Post-scan phases over the full forest: `relate` (derive edges) then `verify`
/// (judge with the graph). Mutates `data` in place: attaches `node.relations`,
/// fills edges / kind_graph / judgment findings.
fn run_graph_phases(ctx: &ComponentRunContext, data: &mut ScanData) -> Result<(), String> {
    let flat = flatten_instances(&data.instances);
    let mut raw_edges: Vec<RawEdge> = Vec::new();
    let mut findings: Vec<ComponentFinding> = Vec::new();

    // Phase 1: relate — index = the forest before edges.
    for component in ctx.components.values() {
        if !component.has_relate {
            continue;
        }
        let targets: Vec<&FlatInstance> = flat
            .iter()
            .filter(|f| f.component_id == component.id)
            .collect();
        if targets.is_empty() {
            continue;
        }
        let host = make_host(ctx, BTreeMap::new())?;
        let lua = codemod_recipe_lua::new_sandbox_with_require(&ctx.codemod_root)
            .map_err(|e| e.to_string())?;
        let index = build_index_facade(&lua, &flat)?;
        for target in targets {
            let Some(value) = invoke_component_hook_with_facade(
                &lua,
                component,
                "relate",
                Rc::clone(&host),
                target.to_json(),
                &index,
            )?
            else {
                continue;
            };
            let (mut edges, mut edge_findings) =
                normalize_raw_edges(hook_to_json(&value)?, &target.id);
            raw_edges.append(&mut edges);
            findings.append(&mut edge_findings);
        }
    }

    let (edges, mut unresolved) = resolve_edges(raw_edges, &flat);
    findings.append(&mut unresolved);
    attach_relations(&mut data.instances, &edges);
    data.kind_graph = kind_graph(&edges, &flat);

    // Phase 2: verify — graph = the forest plus resolved edges.
    let flat = flatten_instances(&data.instances);
    let graph_ready = ctx.components.values().any(|c| c.has_verify)
        && flat.iter().any(|f| {
            ctx.components
                .get(&f.component_id)
                .is_some_and(|c| c.has_verify)
        });
    if graph_ready {
        for component in ctx.components.values() {
            if !component.has_verify {
                continue;
            }
            let targets: Vec<&FlatInstance> = flat
                .iter()
                .filter(|f| f.component_id == component.id)
                .collect();
            if targets.is_empty() {
                continue;
            }
            let host = make_host(ctx, BTreeMap::new())?;
            let lua = codemod_recipe_lua::new_sandbox_with_require(&ctx.codemod_root)
                .map_err(|e| e.to_string())?;
            let graph = build_graph_facade(&lua, &flat, &edges)?;
            for target in targets {
                let Some(value) = invoke_component_hook_with_facade(
                    &lua,
                    component,
                    "verify",
                    Rc::clone(&host),
                    target.to_json(),
                    &graph,
                )?
                else {
                    continue;
                };
                let mut component_findings =
                    normalize_findings(hook_to_json(&value)?, &target.id, &target.locator);
                findings.append(&mut component_findings);
            }
        }
    }

    data.edges = edges;
    data.judgment_findings = findings;
    Ok(())
}

pub(crate) fn scan_component(
    ctx: &ComponentRunContext,
    component: &ComponentDef,
) -> Result<(Vec<ComponentInstance>, Vec<ComponentFinding>), String> {
    let host = make_host(ctx, BTreeMap::new())?;
    let lua = codemod_recipe_lua::new_sandbox_with_require(&ctx.codemod_root)
        .map_err(|e| e.to_string())?;
    let locators = collect_identify_locators(&lua, ctx, component, Rc::clone(&host))?;
    scan_component_contexts(&lua, ctx, component, Rc::clone(&host), locators)
}

/// Re-run identify for a root component (used by incremental detectChanges).
pub(crate) fn identify_locators_for_component(
    ctx: &ComponentRunContext,
    component: &ComponentDef,
) -> Result<Vec<serde_json::Value>, String> {
    let host = make_host(ctx, BTreeMap::new())?;
    let lua = codemod_recipe_lua::new_sandbox_with_require(&ctx.codemod_root)
        .map_err(|e| e.to_string())?;
    collect_identify_locators(&lua, ctx, component, host)
}

fn rescan_root_context(
    ctx: &ComponentRunContext,
    component: &ComponentDef,
    locator: serde_json::Value,
) -> Result<(Vec<ComponentInstance>, Vec<ComponentFinding>), String> {
    let host = make_host(ctx, BTreeMap::new())?;
    let lua = codemod_recipe_lua::new_sandbox_with_require(&ctx.codemod_root)
        .map_err(|e| e.to_string())?;
    scan_component_contexts(&lua, ctx, component, Rc::clone(&host), vec![locator])
}

fn scan_component_contexts(
    lua: &mlua::Lua,
    ctx: &ComponentRunContext,
    component: &ComponentDef,
    host: SharedHost,
    locators: Vec<serde_json::Value>,
) -> Result<(Vec<ComponentInstance>, Vec<ComponentFinding>), String> {
    let mut instances = Vec::new();
    let mut findings = Vec::new();
    for locator in locators {
        let locator_value = normalize_parse_context(locator);
        let parse_raw = invoke_component_hook(
            lua,
            ctx,
            component,
            "parse",
            Rc::clone(&host),
            Some(locator_value.clone()),
        )?;
        let nodes = expand_parse_results(hook_to_json(&parse_raw)?)?;
        for node in nodes {
            let id = instance_id(&component.id, &node, &locator_value);
            findings.extend(soft_validate_parsed(&id, &locator_value, &node));
            let instance_json = serde_json::json!({
                "id": id,
                "componentId": component.id,
                "locator": locator_value,
                "ast": node,
            });
            let summary = try_invoke_component_hook(
                lua,
                ctx,
                component,
                "toSummary",
                Rc::clone(&host),
                Some(instance_json),
            )?
            .map(|v| hook_to_json(&v))
            .transpose()?;
            instances.push(ComponentInstance {
                id,
                component_id: component.id.clone(),
                locator: locator_value.clone(),
                ast: node,
                summary,
            });
        }
    }
    Ok((instances, findings))
}

fn collect_identify_locators(
    lua: &mlua::Lua,
    ctx: &ComponentRunContext,
    component: &ComponentDef,
    host: SharedHost,
) -> Result<Vec<serde_json::Value>, String> {
    let identify_raw =
        invoke_component_hook(lua, ctx, component, "identify", host, None).map_err(|e| {
            if e.contains("missing required hook identify") {
                format!(
                    "stack root component {} requires identify(c) returning a list of locators",
                    component.id
                )
            } else {
                e
            }
        })?;
    let locators = hook_to_json(&identify_raw)?;
    Ok(match locators {
        serde_json::Value::Array(items) => items,
        serde_json::Value::Null => Vec::new(),
        // Empty Luau table `{}` converts to a JSON object, not an array —
        // an empty identify result means zero locators, not one empty locator.
        serde_json::Value::Object(map) if map.is_empty() => Vec::new(),
        other => vec![other],
    })
}

fn identify_diff(
    registry: &RecipeRegistry,
    cache: &ComponentScanCache,
    result: &mut crate::component::cache::DetectResult,
) -> Result<(), String> {
    let Some(stack) = registry.stack() else {
        return Ok(());
    };
    let ctx = ComponentRunContext::from_registry(registry, RunKind::Preview);
    for component_id in &stack.components {
        let Some(component) = registry.get_component(component_id) else {
            continue;
        };
        let locators = identify_locators_for_component(&ctx, component)?;
        let current: Vec<(String, serde_json::Value)> = locators
            .into_iter()
            .map(|locator| {
                let locator = normalize_parse_context(locator);
                (locator_key(&locator), locator)
            })
            .collect();
        let current_keys: BTreeSet<&str> = current.iter().map(|(k, _)| k.as_str()).collect();
        let known: Vec<&crate::component::cache::RootEntry> = cache
            .root_entries
            .values()
            .filter(|e| e.component_id == *component_id)
            .collect();
        let known_keys: BTreeSet<String> = known.iter().map(|e| locator_key(&e.locator)).collect();

        for (key, locator) in &current {
            if !known_keys.contains(key) {
                result.new_roots.push(NewRoot {
                    component_id: component_id.clone(),
                    locator: locator.clone(),
                });
            }
        }
        for entry in known {
            let key = locator_key(&entry.locator);
            if !current_keys.contains(key.as_str()) {
                result.removed_roots.push(RootKey {
                    component_id: entry.component_id.clone(),
                    root_path: entry.root_path.clone(),
                });
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_parse_context_promotes_path_to_source() {
        let ctx = normalize_parse_context(serde_json::json!({ "path": "lib/a.dart" }));
        assert_eq!(ctx["path"], "lib/a.dart");
        assert_eq!(ctx["source"]["path"], "lib/a.dart");
    }

    #[test]
    fn expand_parse_results_skips_null_and_skip_flag() {
        assert!(expand_parse_results(serde_json::Value::Null)
            .unwrap()
            .is_empty());
        assert!(expand_parse_results(serde_json::json!({ "skip": true }))
            .unwrap()
            .is_empty());
    }

    #[test]
    fn expand_parse_results_unwraps_instances_array() {
        let out = expand_parse_results(serde_json::json!({
            "instances": [{ "id": "a" }, { "id": "b" }]
        }))
        .unwrap();
        assert_eq!(out.len(), 2);
    }
}
