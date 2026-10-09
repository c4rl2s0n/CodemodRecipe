//! Relate + verify phases after identify/parse.

use std::collections::BTreeMap;
use std::rc::Rc;

use crate::component::relations::{
    attach_relations, build_graph_facade, build_index_facade, flatten_instances, kind_graph,
    normalize_raw_edges, resolve_edges, FlatInstance, RawEdge,
};
use crate::component::runtime::{
    hook_to_json, invoke_component_hook_with_facade, make_host, ComponentRunContext,
};
use crate::component::scan::types::ScanData;
use crate::component::types::{normalize_findings, ComponentFinding};

pub(super) fn run_graph_phases(ctx: &ComponentRunContext, data: &mut ScanData) -> Result<(), String> {
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

    let member_kinds = crate::component::membership::member_kinds(&ctx.relations);
    let (edges, mut unresolved) = resolve_edges(raw_edges, &flat, &member_kinds);
    findings.append(&mut unresolved);
    let (_membership, mut membership_findings) =
        crate::component::membership::validate_membership_forest(&edges, &member_kinds);
    findings.append(&mut membership_findings);
    attach_relations(&mut data.instances, &edges);
    let architecture: Vec<_> = edges
        .iter()
        .filter(|e| !member_kinds.contains(&e.kind))
        .cloned()
        .collect();
    data.kind_graph = kind_graph(&architecture, &flat);

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

