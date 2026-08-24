//! Luau control-flow gates (replaces YAML step `if` / `ifNot`).

use codemod_recipe_core::apply::file_change::FileChange;
use codemod_recipe_host::engine::runner::{collect_recipe_changes, parse_inline_recipe};
use codemod_recipe_host::lua_runtime::RunKind;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn temp_workspace(name: &str) -> std::path::PathBuf {
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("codemod_step_if_{name}_{n}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    dir
}

fn collect(
    workspace: &std::path::Path,
    source: &str,
    args: &BTreeMap<String, String>,
) -> Vec<FileChange> {
    let recipe =
        parse_inline_recipe(&serde_json::Value::String(source.to_string())).expect("parse");
    let codemod = workspace.join(".codemod");
    std::fs::create_dir_all(&codemod).unwrap();
    let registry = RecipeRegistry::new(workspace.to_path_buf(), codemod);
    collect_recipe_changes(&registry, &recipe, None, args, RunKind::Preview)
        .expect("collect")
        .changes
}

#[test]
fn bool_if_skips_create() {
    let workspace = temp_workspace("bool_skip");
    let source = r#"
return {
  id = "gated_create",
  args = { { name = "includeFile", defaultsTo = "false" } },
  apply = function(c)
    if c.args.includeFile == "true" then
      c:create({
        path = "lib/generated.dart",
        content = "class Generated {}\n",
      })
    end
  end,
}
"#;
    let mut args = BTreeMap::new();
    args.insert("includeFile".to_string(), "false".to_string());
    assert!(collect(&workspace, source, &args).is_empty());

    args.insert("includeFile".to_string(), "true".to_string());
    let changes = collect(&workspace, source, &args);
    assert_eq!(changes.len(), 1);
    assert!(matches!(&changes[0], FileChange::Create { path, .. } if path == "lib/generated.dart"));
}

#[test]
fn file_exists_if_not_skips_when_on_disk() {
    let workspace = temp_workspace("exists_disk");
    std::fs::write(workspace.join("lib/existing.dart"), "class Existing {}\n").unwrap();
    let source = r#"
return {
  id = "gated_by_exists",
  args = { { name = "file", required = true } },
  apply = function(c)
    local function pathExists(path)
      return pcall(function() c:read(path) end)
    end
    if not pathExists(c.args.file) then
      c:create({
        path = "lib/other.dart",
        content = "class Other {}\n",
      })
    end
  end,
}
"#;
    let mut args = BTreeMap::new();
    args.insert("file".to_string(), "lib/existing.dart".to_string());
    assert!(collect(&workspace, source, &args).is_empty());

    args.insert("file".to_string(), "lib/missing.dart".to_string());
    let changes = collect(&workspace, source, &args);
    assert_eq!(changes.len(), 1);
}

#[test]
fn file_exists_sees_prior_create_in_same_recipe() {
    let workspace = temp_workspace("exists_staged");
    let source = r#"
return {
  id = "staged_exists",
  args = { { name = "file", required = true } },
  apply = function(c)
    local function pathExists(path)
      return pcall(function() c:read(path) end)
    end
    c:create({
      path = c.args.file,
      content = "class A {}\n",
    })
    if not pathExists(c.args.file) then
      c:create({
        path = "lib/second.dart",
        content = "class B {}\n",
      })
    end
  end,
}
"#;
    let mut args = BTreeMap::new();
    args.insert("file".to_string(), "lib/first.dart".to_string());
    let changes = collect(&workspace, source, &args);
    assert_eq!(
        changes.len(),
        1,
        "second create should skip after staged first"
    );
    assert!(matches!(&changes[0], FileChange::Create { path, .. } if path == "lib/first.dart"));
}

#[test]
fn conditioned_recipe_ref_skips_subtree() {
    let workspace = temp_workspace("recipe_skip");
    let recipes = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes).unwrap();
    std::fs::write(
        recipes.join("child_create.luau"),
        r#"
return {
  id = "child_create",
  args = { { name = "file", required = true } },
  apply = function(c)
    c:create({
      path = c.args.file,
      content = "class Child {}\n",
    })
  end,
}
"#,
    )
    .unwrap();
    std::fs::write(
        recipes.join("parent_gate.luau"),
        r#"
return {
  id = "parent_gate",
  args = {
    { name = "includeChild", defaultsTo = "false" },
    { name = "file", required = true },
  },
  apply = function(c)
    if c.args.includeChild == "true" then
      c:run("child_create", { file = c.args.file })
    end
  end,
}
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();
    let (recipe, path) = registry.load_recipe_ast("parent_gate").unwrap();

    let mut args = BTreeMap::new();
    args.insert("includeChild".to_string(), "false".to_string());
    args.insert("file".to_string(), "lib/child.dart".to_string());
    let skipped = collect_recipe_changes(
        &registry,
        &recipe,
        Some(path.as_path()),
        &args,
        RunKind::Preview,
    )
    .unwrap()
    .changes;
    assert!(skipped.is_empty());

    args.insert("includeChild".to_string(), "true".to_string());
    let applied = collect_recipe_changes(
        &registry,
        &recipe,
        Some(path.as_path()),
        &args,
        RunKind::Preview,
    )
    .unwrap()
    .changes;
    assert_eq!(applied.len(), 1);
}

#[test]
fn lua_if_gates_multiple_creates() {
    let workspace = temp_workspace("if_group");
    let source = r#"
return {
  id = "gated_group",
  args = { { name = "includeExtra", defaultsTo = "false" } },
  apply = function(c)
    if c.args.includeExtra == "true" then
      c:create({ path = "lib/one.dart", content = "class One {}\n" })
      c:create({ path = "lib/two.dart", content = "class Two {}\n" })
    end
  end,
}
"#;
    let mut args = BTreeMap::new();
    args.insert("includeExtra".to_string(), "false".to_string());
    assert!(collect(&workspace, source, &args).is_empty());

    args.insert("includeExtra".to_string(), "true".to_string());
    let changes = collect(&workspace, source, &args);
    assert_eq!(changes.len(), 2);
}

#[test]
fn nested_lua_if_gates_second_create() {
    let workspace = temp_workspace("if_nested_gates");
    let source = r#"
return {
  id = "nested_gates",
  args = {
    { name = "includeGroup", defaultsTo = "true" },
    { name = "includeSecond", defaultsTo = "false" },
  },
  apply = function(c)
    if c.args.includeGroup == "true" then
      c:create({ path = "lib/one.dart", content = "class One {}\n" })
      if c.args.includeSecond == "true" then
        c:create({ path = "lib/two.dart", content = "class Two {}\n" })
      end
    end
  end,
}
"#;
    let mut args = BTreeMap::new();
    args.insert("includeGroup".to_string(), "true".to_string());
    args.insert("includeSecond".to_string(), "false".to_string());
    let changes = collect(&workspace, source, &args);
    assert_eq!(changes.len(), 1);
    assert!(matches!(&changes[0], FileChange::Create { path, .. } if path == "lib/one.dart"));

    args.insert("includeGroup".to_string(), "false".to_string());
    args.insert("includeSecond".to_string(), "true".to_string());
    assert!(collect(&workspace, source, &args).is_empty());
}

#[test]
fn if_with_recipe_refs_expands_and_gates() {
    let workspace = temp_workspace("if_recipe_group");
    let recipes = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes).unwrap();
    std::fs::write(
        recipes.join("child_a.luau"),
        r#"
return {
  id = "child_a",
  apply = function(c)
    c:create({ path = "lib/a.dart", content = "class A {}\n" })
  end,
}
"#,
    )
    .unwrap();
    std::fs::write(
        recipes.join("child_b.luau"),
        r#"
return {
  id = "child_b",
  apply = function(c)
    c:create({ path = "lib/b.dart", content = "class B {}\n" })
  end,
}
"#,
    )
    .unwrap();
    std::fs::write(
        recipes.join("parent_if.luau"),
        r#"
return {
  id = "parent_if",
  args = { { name = "includeChildren", defaultsTo = "false" } },
  apply = function(c)
    if c.args.includeChildren == "true" then
      c:run("child_a")
      c:run("child_b")
    end
  end,
}
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();
    let (recipe, path) = registry.load_recipe_ast("parent_if").unwrap();

    let mut args = BTreeMap::new();
    args.insert("includeChildren".to_string(), "false".to_string());
    let skipped = collect_recipe_changes(
        &registry,
        &recipe,
        Some(path.as_path()),
        &args,
        RunKind::Preview,
    )
    .unwrap()
    .changes;
    assert!(skipped.is_empty());

    args.insert("includeChildren".to_string(), "true".to_string());
    let applied = collect_recipe_changes(
        &registry,
        &recipe,
        Some(path.as_path()),
        &args,
        RunKind::Preview,
    )
    .unwrap()
    .changes;
    assert_eq!(applied.len(), 2);
}
