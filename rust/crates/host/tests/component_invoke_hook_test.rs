//! componentInvokeHook dry-run / apply: result JSON + transcript (including c:log).

mod common;

use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use common::temp_workspace;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn reference_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../test/fixtures/reference_stack")
}

#[allow(clippy::too_many_arguments)]
fn invoke_hook(
    registry: &mut RecipeRegistry,
    component_id: &str,
    hook: &str,
    preview: bool,
    locator: Option<serde_json::Value>,
    instance: Option<serde_json::Value>,
    operation: Option<String>,
    args: BTreeMap<String, serde_json::Value>,
) -> serde_json::Value {
    dispatch::handle_command(
        registry,
        HostCommand::ComponentInvokeHook {
            component_id: component_id.to_string(),
            hook: hook.to_string(),
            preview,
            locator,
            instance,
            operation,
            args,
        },
    )
}

#[test]
fn identify_dry_run_returns_locators_and_empty_transcript() {
    let root = reference_root();
    let mut registry = RecipeRegistry::new(root.join("workspace"), root.join(".codemod"));
    registry.reload();

    let response = invoke_hook(
        &mut registry,
        "example.entity",
        "identify",
        true,
        None,
        None,
        None,
        BTreeMap::new(),
    );
    assert_eq!(response["ok"], true, "identify failed: {response}");
    let result = response["result"].as_array().expect("locator array");
    assert!(
        result
            .iter()
            .any(|l| l["path"] == "src/entities/recipe.dart"),
        "expected recipe locator, got {result:?}"
    );
}

#[test]
fn parse_verify_dry_run_with_selected_locator_and_instance() {
    let root = reference_root();
    let mut registry = RecipeRegistry::new(root.join("workspace"), root.join(".codemod"));
    registry.reload();

    let locate = invoke_hook(
        &mut registry,
        "example.entity",
        "identify",
        true,
        None,
        None,
        None,
        BTreeMap::new(),
    );
    let locator = locate["result"].as_array().unwrap()[0].clone();

    let parsed = invoke_hook(
        &mut registry,
        "example.entity",
        "parse",
        true,
        Some(locator.clone()),
        None,
        None,
        BTreeMap::new(),
    );
    assert_eq!(parsed["ok"], true, "parse failed: {parsed}");
    assert_eq!(parsed["result"]["id"], "recipe");

    let instance = serde_json::json!({
        "id": "example.entity/recipe",
        "componentId": "example.entity",
        "locator": locator,
        "ast": parsed["result"],
    });
    let verified = invoke_hook(
        &mut registry,
        "example.entity",
        "verify",
        true,
        None,
        Some(instance),
        None,
        BTreeMap::new(),
    );
    assert_eq!(verified["ok"], true, "verify failed: {verified}");
    // Empty Lua arrays serialize as objects; non-empty as arrays.
    assert!(
        verified["result"].is_array() || verified["result"].is_object(),
        "expected findings JSON, got {}",
        verified["result"]
    );
}

#[test]
fn dry_run_captures_log_and_partial_transcript_on_failure() {
    let workspace = temp_workspace("hook_log_transcript");
    let codemod = workspace.join(".codemod");
    fs::create_dir_all(codemod.join("components")).unwrap();
    fs::write(
        codemod.join("stack.luau"),
        r#"
return {
  id = "log-stack",
  components = { "log.comp" },
}
"#,
    )
    .unwrap();
    fs::write(
        codemod.join("components/log.comp.luau"),
        r#"
return {
  id = "log.comp",
  identify = function(c)
    c:log("identify-start", { phase = "identify" })
    return { { path = "missing.dart", name = "x" } }
  end,
  parse = function(c, locator)
    c:log("before-fail", { path = locator.path })
    error("parse boom")
  end,
  verify = function(_c, _instance)
    return {}
  end,
}
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace, codemod);
    registry.reload();

    let identify = invoke_hook(
        &mut registry,
        "log.comp",
        "identify",
        true,
        None,
        None,
        None,
        BTreeMap::new(),
    );
    assert_eq!(identify["ok"], true, "{identify}");
    let empty: Vec<serde_json::Value> = vec![];
    let kinds: Vec<&str> = identify["transcript"]
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(|e| e["kind"].as_str())
        .collect();
    assert!(
        kinds.contains(&"log"),
        "expected log in transcript, got {kinds:?}"
    );

    let parse = invoke_hook(
        &mut registry,
        "log.comp",
        "parse",
        true,
        Some(serde_json::json!({ "path": "missing.dart", "name": "x" })),
        None,
        None,
        BTreeMap::new(),
    );
    assert_eq!(parse["ok"], false, "expected parse failure: {parse}");
    let fail_kinds: Vec<&str> = parse["transcript"]
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(|e| e["kind"].as_str())
        .collect();
    assert!(
        fail_kinds.contains(&"log"),
        "expected partial log transcript on failure, got {fail_kinds:?} err={}",
        parse["error"]
    );
}

#[test]
fn scaffold_preview_vs_apply_via_invoke_hook() {
    let root = reference_root();
    let workspace = temp_workspace("hook_scaffold");
    let codemod_src = root.join(".codemod");
    let workspace_src = root.join("workspace");
    copy_dir(&codemod_src, &workspace.join(".codemod"));
    copy_dir(&workspace_src, &workspace);

    // Use flutter_poc style? reference_stack may not have scaffold.
    // Write a tiny scaffold component into the temp stack.
    fs::write(
        workspace.join(".codemod/components/example.scaffold.luau"),
        r#"
return {
  id = "example.scaffold",
  parse = function(_c, _locator) return nil end,
  verify = function(_c, _instance) return {} end,
  scaffold = {
    args = { { name = "name", required = true } },
    apply = function(c, args)
      c:log("scaffolding", { name = args.name })
      c:create({
        path = "src/entities/" .. args.name .. ".dart",
        text = "class " .. args.name .. " {}\n",
      })
    end,
  },
}
"#,
    )
    .unwrap();
    fs::write(
        workspace.join(".codemod/stack.luau"),
        r#"
return {
  id = "reference-stack",
  components = { "example.entity", "example.store" },
}
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("name".to_string(), serde_json::json!("Widget"));

    let preview = invoke_hook(
        &mut registry,
        "example.scaffold",
        "scaffold",
        true,
        None,
        None,
        None,
        args.clone(),
    );
    assert_eq!(preview["ok"], true, "scaffold preview failed: {preview}");
    assert!(
        preview["files"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        "expected files in preview"
    );
    assert!(
        !workspace.join("src/entities/Widget.dart").exists(),
        "preview must not flush"
    );
    let empty: Vec<serde_json::Value> = vec![];
    let kinds: Vec<&str> = preview["transcript"]
        .as_array()
        .unwrap_or(&empty)
        .iter()
        .filter_map(|e| e["kind"].as_str())
        .collect();
    assert!(kinds.contains(&"log"));

    let apply = invoke_hook(
        &mut registry,
        "example.scaffold",
        "scaffold",
        false,
        None,
        None,
        None,
        args,
    );
    assert_eq!(apply["ok"], true, "scaffold apply failed: {apply}");
    assert!(
        workspace.join("src/entities/Widget.dart").exists(),
        "apply must flush"
    );
}

#[test]
fn describe_component_reports_hook_flags() {
    let root = reference_root();
    let mut registry = RecipeRegistry::new(root.join("workspace"), root.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::DescribeComponent {
            component_id: "example.entity".to_string(),
        },
    );
    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(response["component"]["hasIdentify"], true);
    assert_eq!(response["component"]["hasToSummary"], false);
    assert_eq!(response["component"]["hasScaffold"], false);
}

fn copy_dir(src: &std::path::Path, dst: &std::path::Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &to);
        } else {
            fs::copy(entry.path(), to).unwrap();
        }
    }
}
