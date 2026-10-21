//! Identify + parse roots into instances.

use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use codemod_recipe_lua::model::ComponentDef;

use crate::component::cache::{locator_key, ComponentScanCache, NewRoot, RootKey};
use crate::component::runtime::{
    hook_to_json, invoke_component_hook, make_host, try_invoke_component_hook, ComponentRunContext,
};
use crate::component::types::{
    expand_parse_results, instance_id, normalize_parse_context, soft_validate_parsed,
    ComponentFinding, ComponentInstance,
};
use crate::host_api::context::SharedHost;
use crate::lua_runtime::RunKind;
use crate::recipe::registry::RecipeRegistry;

pub(crate) fn scan_component(
    ctx: &ComponentRunContext,
    component: &ComponentDef,
) -> Result<(Vec<ComponentInstance>, Vec<ComponentFinding>), String> {
    let host = make_host(ctx, BTreeMap::new())?;
    let lua = codemod_recipe_lua::new_sandbox_with_require(&ctx.codemod_root)
        .map_err(|e| e.to_string())?;
    let locators = collect_identify_locators(&lua, ctx, component, Rc::clone(&host))?;
    let mut empty_findings = Vec::new();
    if locators.is_empty() {
        empty_findings.push(ComponentFinding {
            severity: "error".to_string(),
            code: "composer.identify_empty".to_string(),
            message: format!(
                "stack component '{}' identify() returned no locators — path drift or missing sources?",
                component.id
            ),
            instance_id: None,
            locator: None,
        });
    }
    let (instances, mut findings) =
        scan_component_contexts(&lua, ctx, component, Rc::clone(&host), locators)?;
    empty_findings.append(&mut findings);
    Ok((instances, empty_findings))
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

pub(super) fn rescan_root_context(
    ctx: &ComponentRunContext,
    component: &ComponentDef,
    locator: serde_json::Value,
) -> Result<(Vec<ComponentInstance>, Vec<ComponentFinding>), String> {
    let host = make_host(ctx, BTreeMap::new())?;
    let lua = codemod_recipe_lua::new_sandbox_with_require(&ctx.codemod_root)
        .map_err(|e| e.to_string())?;
    scan_component_contexts(&lua, ctx, component, Rc::clone(&host), vec![locator])
}

pub(super) fn scan_component_contexts(
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

pub(super) fn collect_identify_locators(
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

pub(super) fn identify_diff(
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
