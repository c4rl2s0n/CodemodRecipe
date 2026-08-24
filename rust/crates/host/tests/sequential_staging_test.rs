//! Sequential staging: create→edit, edit→edit, skip+edit, conflicts.

use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static WORKSPACE_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_workspace(name: &str) -> PathBuf {
    let n = WORKSPACE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("seq_stage_{name}_{}_{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join(".codemod/recipes")).unwrap();
    dir
}

fn preview_inline(
    registry: &mut RecipeRegistry,
    inline: serde_json::Value,
    args: BTreeMap<String, String>,
) -> serde_json::Value {
    dispatch::handle_command(
        registry,
        HostCommand::Preview {
            recipe: None,
            inline_recipe: Some(inline),
            args,
            snippet_lines: Some(10),
        },
    )
}

#[test]
fn create_skip_then_create_skip_then_edit_is_single_create() {
    let workspace = temp_workspace("create_skip_create_edit");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!(
        r#"
return {
  id = "ensure_barrel_twice",
  apply = function(c)
    c:create({ path = "lib/barrel.dart", content = "// barrel\n", ifExists = "skip" })
    c:create({ path = "lib/barrel.dart", content = "// unused ensure\n", ifExists = "skip" })
    c:insert({
      path = "lib/barrel.dart",
      query = "(program) @root",
      capture = "root",
      anchor = "end",
      text = "export 'foo.dart';\n",
    })
  end,
}
"#
    );

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["kind"], "create");
    assert_eq!(files[0]["path"], "lib/barrel.dart");
    assert!(files[0]["isNew"].as_bool().unwrap());
    let snippet = files[0]["snippet"].as_str().unwrap_or("");
    assert!(
        snippet.contains("export 'foo.dart'") && snippet.contains("// barrel"),
        "snippet={snippet}"
    );

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn create_then_edit_missing_file_is_single_create() {
    let workspace = temp_workspace("create_edit_new");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!(
        r#"
return {
  id = "ensure_barrel",
  apply = function(c)
    c:create({ path = "lib/barrel.dart", content = "// barrel\n", ifExists = "skip" })
    c:insert({
      path = "lib/barrel.dart",
      query = "(program) @root",
      capture = "root",
      anchor = "end",
      text = "export 'foo.dart';\n",
    })
  end,
}
"#
    );

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["kind"], "create");
    assert_eq!(files[0]["path"], "lib/barrel.dart");
    assert!(files[0]["isNew"].as_bool().unwrap());
    let snippet = files[0]["snippet"].as_str().unwrap_or("");
    assert!(
        snippet.contains("export") || snippet.contains("barrel"),
        "snippet={snippet}"
    );

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn create_skip_then_edit_existing_is_patch() {
    let workspace = temp_workspace("create_skip_edit");
    std::fs::write(workspace.join("lib/barrel.dart"), "// barrel\n").unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!(
        r#"
return {
  id = "ensure_barrel",
  apply = function(c)
    c:create({ path = "lib/barrel.dart", content = "// unused\n", ifExists = "skip" })
    c:insert({
      path = "lib/barrel.dart",
      query = "(program) @root",
      capture = "root",
      anchor = "end",
      text = "export 'foo.dart';\n",
    })
  end,
}
"#
    );

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["kind"], "edit");
    assert_eq!(files[0]["path"], "lib/barrel.dart");
    assert!(!files[0]["isNew"].as_bool().unwrap());
    assert!(files[0]["original"].as_str().is_some());
    assert!(files[0]["modified"].as_str().is_some());

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn edit_then_edit_dependent_class_and_constructor() {
    let workspace = temp_workspace("edit_edit");
    std::fs::write(workspace.join("lib/foo.dart"), "class Foo {\n}\n").unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!(
        r#"
return {
  id = "class_then_ctor",
  apply = function(c)
    c:insert({
      path = "lib/foo.dart",
      query = "(class_definition name: (identifier) @n body: (class_body) @body (#eq? @n \"Foo\"))",
      capture = "body",
      anchor = "end",
      text = "  void bar() {}\n",
    })
    c:insert({
      path = "lib/foo.dart",
      query = "(class_definition name: (identifier) @className body: (class_body (method_signature (function_signature name: (identifier) @methodName)) @member) (#eq? @className \"Foo\") (#eq? @methodName \"bar\"))",
      capture = "member",
      anchor = "end",
      text = "\n  Foo();",
    })
  end,
}
"#
    );

    let response = preview_inline(&mut registry, inline.clone(), BTreeMap::new());
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["kind"], "edit");

    // Apply and verify disk content
    let token = response["previewToken"].as_str().unwrap();
    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::Apply {
            recipe: None,
            inline_recipe: Some(inline),
            args: BTreeMap::new(),
            preview_token: token.to_string(),
            selection: serde_json::json!({}),
        },
    );
    assert_eq!(apply["ok"], true, "{}", apply["error"]);
    let src = std::fs::read_to_string(workspace.join("lib/foo.dart")).unwrap();
    assert!(src.contains("void bar()"), "{src}");
    assert!(src.contains("Foo();"), "{src}");

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn create_fail_when_exists() {
    let workspace = temp_workspace("create_fail");
    std::fs::write(workspace.join("lib/a.dart"), "x\n").unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!(
        r#"
return {
  id = "fail_create",
  apply = function(c)
    c:create({ path = "lib/a.dart", content = "y\n", ifExists = "fail" })
  end,
}
"#
    );

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], false);
    assert!(
        response["error"]
            .as_str()
            .unwrap_or("")
            .contains("already exists"),
        "{}",
        response["error"]
    );

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn edit_then_create_same_path_errors() {
    let workspace = temp_workspace("edit_then_create");
    std::fs::write(workspace.join("lib/a.dart"), "class A {}\n").unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!(
        r#"
return {
  id = "bad_order",
  apply = function(c)
    c:insert({
      path = "lib/a.dart",
      query = "(class_definition name: (identifier) @n body: (class_body) @body (#eq? @n \"A\"))",
      capture = "body",
      anchor = "end",
      text = "  // x\n",
    })
    c:create({ path = "lib/a.dart", content = "class A {}\n", ifExists = "fail" })
  end,
}
"#
    );

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], false);
    assert!(
        response["error"]
            .as_str()
            .unwrap_or("")
            .contains("already exists"),
        "{}",
        response["error"]
    );

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn create_and_edit_different_paths() {
    let workspace = temp_workspace("multi_path");
    std::fs::write(workspace.join("lib/existing.dart"), "class E {\n}\n").unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!(
        r#"
return {
  id = "multi",
  apply = function(c)
    c:create({ path = "lib/new.dart", content = "class N {}\n", ifExists = "fail" })
    c:insert({
      path = "lib/existing.dart",
      query = "(class_definition name: (identifier) @n body: (class_body) @body (#eq? @n \"E\"))",
      capture = "body",
      anchor = "end",
      text = "  // patched\n",
    })
  end,
}
"#
    );

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    let kinds: Vec<_> = files.iter().map(|f| f["kind"].as_str().unwrap()).collect();
    assert!(kinds.contains(&"create"));
    assert!(kinds.contains(&"edit"));

    let _ = std::fs::remove_dir_all(workspace);
}
