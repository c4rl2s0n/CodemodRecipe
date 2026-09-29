//! Catalog + runtime `require` for Library helpers (id = asset intent).

mod common;

use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use common::temp_workspace;
use std::collections::BTreeMap;

#[test]
fn library_modules_are_not_catalogued() {
    let workspace = temp_workspace("require_lib_skip");
    let codemod = workspace.join(".codemod");
    std::fs::create_dir_all(codemod.join("recipes")).unwrap();
    std::fs::write(
        codemod.join("helpers.luau"),
        r#"return {
  ping = function() return "pong" end,
}
"#,
    )
    .unwrap();
    std::fs::write(
        codemod.join("recipes/demo.luau"),
        r#"return {
  id = "demo.require",
  apply = function(c) end,
}
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), codemod);
    registry.reload();
    assert!(
        registry.diagnostics().is_empty(),
        "{:?}",
        registry.diagnostics()
    );
    let list = dispatch::handle_command(&mut registry, HostCommand::List);
    let recipes = list["recipes"].as_array().unwrap();
    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0]["id"], "demo.require");

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn id_without_valid_shape_is_diagnostic() {
    let workspace = temp_workspace("require_malformed_id");
    let codemod = workspace.join(".codemod");
    std::fs::create_dir_all(&codemod).unwrap();
    std::fs::write(
        codemod.join("broken.luau"),
        r#"return { id = "broken.only" }
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), codemod);
    registry.reload();
    assert!(
        registry
            .diagnostics()
            .iter()
            .any(|d| d.message.contains("unknown asset shape")),
        "{:?}",
        registry.diagnostics()
    );

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn apply_without_id_is_diagnostic() {
    let workspace = temp_workspace("require_apply_no_id");
    let codemod = workspace.join(".codemod");
    std::fs::create_dir_all(&codemod).unwrap();
    std::fs::write(
        codemod.join("bad.luau"),
        r#"return { apply = function(c) end }
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), codemod);
    registry.reload();
    assert!(
        registry
            .diagnostics()
            .iter()
            .any(|d| d.message.contains("requires top-level id")),
        "{:?}",
        registry.diagnostics()
    );

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn preview_can_require_helper_under_codemod() {
    let workspace = temp_workspace("require_preview");
    std::fs::create_dir_all(workspace.join("lib")).unwrap();
    std::fs::write(workspace.join("lib/main.dart"), "void main() {}\n").unwrap();
    let codemod = workspace.join(".codemod");
    std::fs::create_dir_all(codemod.join("recipes")).unwrap();
    std::fs::write(
        codemod.join("str.luau"),
        r#"return {
  twice = function(s) return s .. s end,
}
"#,
    )
    .unwrap();
    std::fs::write(
        codemod.join("recipes/use_str.luau"),
        r#"
local str = require("../str")
return {
  id = "use_str",
  apply = function(c, args)
    local text = str.twice("-- x\n")
    c:insert({
      path = "lib/main.dart",
      query = "(program) @root",
      capture = "root",
      anchor = "start",
      text = text,
    })
  end,
}
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), codemod);
    registry.reload();
    assert!(
        registry.diagnostics().is_empty(),
        "{:?}",
        registry.diagnostics()
    );

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("use_str".into()),
            inline_recipe: None,
            args: BTreeMap::new(),
            snippet_lines: None,
        },
    );
    assert_eq!(response["ok"], true, "{response}");
    assert!(
        response["files"]
            .as_array()
            .map(|a| !a.is_empty())
            .unwrap_or(false),
        "{response}"
    );

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn preview_cannot_require_another_recipe() {
    let workspace = temp_workspace("require_reject_recipe");
    std::fs::create_dir_all(workspace.join("lib")).unwrap();
    std::fs::write(workspace.join("lib/main.dart"), "void main() {}\n").unwrap();
    let codemod = workspace.join(".codemod");
    std::fs::create_dir_all(codemod.join("recipes")).unwrap();
    std::fs::write(
        codemod.join("recipes/other.luau"),
        r#"return {
  id = "other",
  apply = function(c) end,
}
"#,
    )
    .unwrap();
    std::fs::write(
        codemod.join("recipes/bad_req.luau"),
        r#"
local other = require("./other")
return {
  id = "bad_req",
  apply = function(c) end,
}
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), codemod);
    registry.reload();

    let cannot_require = registry
        .diagnostics()
        .iter()
        .any(|d| d.message.contains("cannot require") || d.message.contains("Recipe"));
    if cannot_require {
        // Catalog eval runs top-level require — rejected at load time.
        let _ = std::fs::remove_dir_all(workspace);
        return;
    }

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("bad_req".into()),
            inline_recipe: None,
            args: BTreeMap::new(),
            snippet_lines: None,
        },
    );
    assert_eq!(response["ok"], false, "{response}");
    let msg = response["error"].as_str().unwrap_or_default();
    assert!(
        msg.contains("cannot require") || msg.contains("Recipe") || msg.contains("unknown"),
        "{response}"
    );

    let _ = std::fs::remove_dir_all(workspace);
}
