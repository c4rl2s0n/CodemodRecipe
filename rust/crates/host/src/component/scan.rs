use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use codemod_recipe_lua::model::ComponentDef;

use crate::component::cache::ComponentScanCache;
use crate::component::runtime::{
    hook_to_json, invoke_component_hook, make_host, try_invoke_component_hook, ComponentRunContext,
};
use crate::component::types::{
    expand_parse_results, instance_id, normalize_findings, normalize_parse_context,
    soft_validate_parsed, ComponentFinding, ComponentInstance, ComponentScanResponse,
};
use crate::host_api::context::SharedHost;
use crate::lua_runtime::RunKind;
use crate::recipe::registry::RecipeRegistry;
use crate::sandbox::workspace_glob::glob_workspace;

#[derive(Debug, Clone, Default)]
pub struct ComponentIncrementalScanRequest {
    pub dirty_paths: Vec<String>,
    pub detect_changes: bool,
    pub force: bool,
}

pub fn component_scan(registry: &mut RecipeRegistry) -> ComponentScanResponse {
    match run_full_scan(registry) {
        Ok((response, instances, findings)) => {
            *registry.component_scan_cache_mut() =
                Some(ComponentScanCache::from_scan(registry, instances, findings));
            response
        }
        Err(response) => response,
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
        match cache.detect_changes(registry, &request.dirty_paths) {
            Ok(detect) => {
                new_roots = detect.new_roots;
                removed_roots = detect.removed_roots;
                roots_to_reparse.extend(detect.roots_to_reparse);
            }
            Err(message) => return scan_error(&message),
        }
    } else {
        roots_to_reparse.extend(cache.roots_for_dirty_paths(&request.dirty_paths));
    }

    if roots_to_reparse.is_empty() && new_roots.is_empty() && removed_roots.is_empty() {
        return cached_response(registry, &cache);
    }

    let ctx = ComponentRunContext::from_registry(registry, RunKind::Preview);
    let mut updated_instances = Vec::new();
    let mut updated_findings = Vec::new();

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
                updated_findings.append(&mut findings);
            }
            Err(message) => return scan_error(&message),
        }
    }

    for root in &new_roots {
        let Some(component) = registry.get_component(&root.component_id) else {
            continue;
        };
        let context = normalize_parse_context(serde_json::json!({
            "path": root.root_path,
            "source": { "path": root.root_path },
        }));
        match rescan_root_context(&ctx, component, context) {
            Ok((mut instances, mut findings)) => {
                updated_instances.append(&mut instances);
                updated_findings.append(&mut findings);
            }
            Err(message) => return scan_error(&message),
        }
    }

    let mut cache = cache;
    cache.merge_rescan(
        registry,
        &removed_roots,
        updated_instances,
        updated_findings,
    );
    let response = cached_response(registry, &cache);
    *registry.component_scan_cache_mut() = Some(cache);
    response
}

fn cached_response(registry: &RecipeRegistry, cache: &ComponentScanCache) -> ComponentScanResponse {
    ComponentScanResponse {
        ok: true,
        error: None,
        instances: Some(cache.instances.clone()),
        findings: Some(cache.findings.clone()),
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
        diagnostics: None,
    }
}

fn run_full_scan(
    registry: &RecipeRegistry,
) -> Result<(ComponentScanResponse, Vec<ComponentInstance>, Vec<ComponentFinding>), ComponentScanResponse>
{
    if registry.components().is_empty() {
        return Ok((
            ComponentScanResponse {
                ok: true,
                error: None,
                instances: Some(Vec::new()),
                findings: Some(Vec::new()),
                diagnostics: None,
            },
            Vec::new(),
            Vec::new(),
        ));
    }
    let Some(stack) = registry.stack() else {
        return Err(ComponentScanResponse {
            ok: false,
            error: Some("stack manifest is required when components are loaded".to_string()),
            instances: None,
            findings: None,
            diagnostics: Some(registry.diagnostics().to_vec()),
        });
    };
    let ctx = ComponentRunContext::from_registry(registry, RunKind::Preview);
    let mut instances = Vec::new();
    let mut findings = Vec::new();
    for component_id in &stack.components {
        let Some(component) = registry.get_component(component_id) else {
            continue;
        };
        let root_glob = stack.globs.get(component_id).map(String::as_str);
        match scan_component(&ctx, component, root_glob) {
            Ok((mut found, mut component_findings)) => {
                instances.append(&mut found);
                findings.append(&mut component_findings);
            }
            Err(message) => {
                return Err(ComponentScanResponse {
                    ok: false,
                    error: Some(message),
                    instances: None,
                    findings: None,
                    diagnostics: Some(registry.diagnostics().to_vec()),
                });
            }
        }
    }
    Ok((
        ComponentScanResponse {
            ok: true,
            error: None,
            instances: Some(instances.clone()),
            findings: Some(findings.clone()),
            diagnostics: if registry.diagnostics().is_empty() {
                None
            } else {
                Some(registry.diagnostics().to_vec())
            },
        },
        instances,
        findings,
    ))
}

pub(crate) fn scan_component(
    ctx: &ComponentRunContext,
    component: &ComponentDef,
    root_glob: Option<&str>,
) -> Result<(Vec<ComponentInstance>, Vec<ComponentFinding>), String> {
    let host = make_host(ctx, BTreeMap::new())?;
    let lua = codemod_recipe_lua::new_sandbox().map_err(|e| e.to_string())?;
    let contexts = collect_parse_contexts(&lua, ctx, component, Rc::clone(&host), root_glob)?;
    scan_component_contexts(&lua, ctx, component, Rc::clone(&host), contexts)
}

fn rescan_root_context(
    ctx: &ComponentRunContext,
    component: &ComponentDef,
    context: serde_json::Value,
) -> Result<(Vec<ComponentInstance>, Vec<ComponentFinding>), String> {
    let host = make_host(ctx, BTreeMap::new())?;
    let lua = codemod_recipe_lua::new_sandbox().map_err(|e| e.to_string())?;
    scan_component_contexts(
        &lua,
        ctx,
        component,
        Rc::clone(&host),
        vec![context],
    )
}

fn scan_component_contexts(
    lua: &mlua::Lua,
    ctx: &ComponentRunContext,
    component: &ComponentDef,
    host: SharedHost,
    contexts: Vec<serde_json::Value>,
) -> Result<(Vec<ComponentInstance>, Vec<ComponentFinding>), String> {
    let mut instances = Vec::new();
    let mut findings = Vec::new();
    for context in contexts {
        let context_value = normalize_parse_context(context);
        if !should_parse_after_identify(
            lua,
            ctx,
            component,
            Rc::clone(&host),
            context_value.clone(),
        )? {
            continue;
        }
        let parse_raw = invoke_component_hook(
            lua,
            ctx,
            component,
            "parse",
            Rc::clone(&host),
            Some(context_value.clone()),
        )?;
        let parsed_values = expand_parse_results(hook_to_json(&parse_raw)?)?;
        for parsed in parsed_values {
            let id = instance_id(&component.id, &parsed, &context_value);
            findings.extend(soft_validate_parsed(&id, &context_value, &parsed));
            let instance_json = serde_json::json!({
                "id": id,
                "componentId": component.id,
                "locator": context_value,
                "parsed": parsed,
            });
            let verify_raw = invoke_component_hook(
                lua,
                ctx,
                component,
                "verify",
                Rc::clone(&host),
                Some(instance_json),
            )?;
            let verify_json = hook_to_json(&verify_raw)?;
            let mut component_findings = normalize_findings(verify_json, &id, &context_value);
            findings.append(&mut component_findings);
            let instance_for_summary = serde_json::json!({
                "id": id,
                "componentId": component.id,
                "locator": context_value,
                "parsed": parsed,
            });
            let summary = try_invoke_component_hook(
                lua,
                ctx,
                component,
                "toSummary",
                Rc::clone(&host),
                Some(instance_for_summary),
            )?
            .map(|v| hook_to_json(&v))
            .transpose()?;
            instances.push(ComponentInstance {
                id,
                component_id: component.id.clone(),
                locator: context_value.clone(),
                parsed,
                summary,
            });
        }
    }
    Ok((instances, findings))
}

fn collect_parse_contexts(
    lua: &mlua::Lua,
    ctx: &ComponentRunContext,
    component: &ComponentDef,
    host: SharedHost,
    root_glob: Option<&str>,
) -> Result<Vec<serde_json::Value>, String> {
    if let Some(discover_raw) =
        try_invoke_component_hook(lua, ctx, component, "discover", Rc::clone(&host), None)?
    {
        let locators = hook_to_json(&discover_raw)?;
        return Ok(match locators {
            serde_json::Value::Array(items) => items,
            serde_json::Value::Null => Vec::new(),
            other => vec![other],
        });
    }
    if let Some(pattern) = root_glob {
        let paths = glob_workspace(&ctx.workspace_root, pattern)?;
        return Ok(paths
            .into_iter()
            .map(|path| serde_json::json!({ "path": path, "source": { "path": path } }))
            .collect());
    }
    Ok(Vec::new())
}

fn should_parse_after_identify(
    lua: &mlua::Lua,
    ctx: &ComponentRunContext,
    component: &ComponentDef,
    host: SharedHost,
    context: serde_json::Value,
) -> Result<bool, String> {
    let Some(identify_raw) =
        try_invoke_component_hook(lua, ctx, component, "identify", host, Some(context))?
    else {
        return Ok(true);
    };
    Ok(match hook_to_json(&identify_raw)? {
        serde_json::Value::Bool(b) => b,
        serde_json::Value::Null => false,
        other => !other.is_null(),
    })
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
        assert!(expand_parse_results(serde_json::Value::Null).unwrap().is_empty());
        assert!(
            expand_parse_results(serde_json::json!({ "skip": true }))
                .unwrap()
                .is_empty()
        );
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
