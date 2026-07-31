//! Step-level `if` / `ifNot` integration.

use codemod_recipe_core::file_change::FileChange;
use codemod_recipe_engine::engine::parse_recipe_yaml;
use codemod_recipe_host::registry::RecipeRegistry;
use codemod_recipe_host::runner::collect_recipe_changes;
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
    yaml: &str,
    args: &BTreeMap<String, String>,
) -> Vec<FileChange> {
    let recipe = parse_recipe_yaml(yaml).expect("parse");
    let codemod = workspace.join(".codemod");
    std::fs::create_dir_all(&codemod).unwrap();
    let registry = RecipeRegistry::new(workspace.to_path_buf(), codemod);
    collect_recipe_changes(&registry, &recipe, None, args)
        .expect("collect")
        .changes
}

#[test]
fn bool_if_skips_create() {
    let workspace = temp_workspace("bool_skip");
    let yaml = r#"
id: gated_create
args:
  - name: includeFile
    defaultsTo: "false"
steps:
  - create:
      path: "lib/generated.dart"
      template: "class Generated {}\n"
      if: includeFile
"#;
    let mut args = BTreeMap::new();
    args.insert("includeFile".to_string(), "false".to_string());
    assert!(collect(&workspace, yaml, &args).is_empty());

    args.insert("includeFile".to_string(), "true".to_string());
    let changes = collect(&workspace, yaml, &args);
    assert_eq!(changes.len(), 1);
    assert!(matches!(&changes[0], FileChange::Create { path, .. } if path == "lib/generated.dart"));
}

#[test]
fn file_exists_if_not_skips_when_on_disk() {
    let workspace = temp_workspace("exists_disk");
    std::fs::write(workspace.join("lib/existing.dart"), "class Existing {}\n").unwrap();
    let yaml = r#"
id: gated_by_exists
args:
  - name: file
    required: true
steps:
  - create:
      path: "lib/other.dart"
      template: "class Other {}\n"
      ifNot: file | file_exists
"#;
    let mut args = BTreeMap::new();
    args.insert("file".to_string(), "lib/existing.dart".to_string());
    assert!(collect(&workspace, yaml, &args).is_empty());

    args.insert("file".to_string(), "lib/missing.dart".to_string());
    let changes = collect(&workspace, yaml, &args);
    assert_eq!(changes.len(), 1);
}

#[test]
fn file_exists_sees_prior_create_in_same_recipe() {
    let workspace = temp_workspace("exists_staged");
    let yaml = r#"
id: staged_exists
args:
  - name: file
    required: true
steps:
  - create:
      path: "{{ file }}"
      template: "class A {}\n"
  - create:
      path: "lib/second.dart"
      template: "class B {}\n"
      ifNot: file | file_exists
"#;
    let mut args = BTreeMap::new();
    args.insert("file".to_string(), "lib/first.dart".to_string());
    let changes = collect(&workspace, yaml, &args);
    assert_eq!(changes.len(), 1, "second create should skip after staged first");
    assert!(matches!(&changes[0], FileChange::Create { path, .. } if path == "lib/first.dart"));
}

#[test]
fn conditioned_recipe_ref_skips_subtree() {
    let workspace = temp_workspace("recipe_skip");
    let recipes = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes).unwrap();
    std::fs::write(
        recipes.join("child_create.yaml"),
        r#"
id: child_create
args:
  - name: file
    required: true
steps:
  - create:
      path: "{{ file }}"
      template: "class Child {}\n"
"#,
    )
    .unwrap();
    std::fs::write(
        recipes.join("parent_gate.yaml"),
        r#"
id: parent_gate
args:
  - name: includeChild
    defaultsTo: "false"
  - name: file
    required: true
steps:
  - recipe:
      id: child_create
      if: includeChild
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();
    let (recipe, path) = registry.load_recipe_ast("parent_gate").unwrap();

    let mut args = BTreeMap::new();
    args.insert("includeChild".to_string(), "false".to_string());
    args.insert("file".to_string(), "lib/child.dart".to_string());
    let skipped = collect_recipe_changes(&registry, &recipe, Some(path.as_path()), &args)
        .unwrap()
        .changes;
    assert!(skipped.is_empty());

    args.insert("includeChild".to_string(), "true".to_string());
    let applied = collect_recipe_changes(&registry, &recipe, Some(path.as_path()), &args)
        .unwrap()
        .changes;
    assert_eq!(applied.len(), 1);
}

#[test]
fn edit_if_and_when_not_both_required() {
    let workspace = temp_workspace("edit_if_when");
    std::fs::write(workspace.join("lib/widget.dart"), "class Widget {\n}\n").unwrap();
    let yaml = r#"
id: edit_gated
args:
  - name: file
    required: true
  - name: migrateLegacy
    defaultsTo: "true"
steps:
  - edit:
      path: "{{ file }}"
      language: dart
      if: migrateLegacy
      whenNot:
        - |
          (class_definition
            body: (class_body
              (declaration
                (initialized_identifier_list
                  (initialized_identifier
                    (identifier) @fieldName)))
              (#eq? @fieldName "migrated")))
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: "\n// migrated\n"
"#;
    let mut args = BTreeMap::new();
    args.insert("file".to_string(), "lib/widget.dart".to_string());
    args.insert("migrateLegacy".to_string(), "false".to_string());
    assert!(collect(&workspace, yaml, &args).is_empty());

    args.insert("migrateLegacy".to_string(), "true".to_string());
    let changes = collect(&workspace, yaml, &args);
    assert_eq!(changes.len(), 1);
}

#[test]
fn if_step_group_gates_all_children() {
    let workspace = temp_workspace("if_group");
    let yaml = r#"
id: gated_group
args:
  - name: includeExtra
    defaultsTo: "false"
steps:
  - if:
      if: includeExtra
      steps:
        - create:
            path: "lib/one.dart"
            template: "class One {}\n"
        - create:
            path: "lib/two.dart"
            template: "class Two {}\n"
"#;
    let mut args = BTreeMap::new();
    args.insert("includeExtra".to_string(), "false".to_string());
    assert!(collect(&workspace, yaml, &args).is_empty());

    args.insert("includeExtra".to_string(), "true".to_string());
    let changes = collect(&workspace, yaml, &args);
    assert_eq!(changes.len(), 2);
}

#[test]
fn if_step_outer_and_child_if_both_apply() {
    let workspace = temp_workspace("if_nested_gates");
    let yaml = r#"
id: nested_gates
args:
  - name: includeGroup
    defaultsTo: "true"
  - name: includeSecond
    defaultsTo: "false"
steps:
  - if:
      if: includeGroup
      steps:
        - create:
            path: "lib/one.dart"
            template: "class One {}\n"
        - create:
            path: "lib/two.dart"
            template: "class Two {}\n"
            if: includeSecond
"#;
    let mut args = BTreeMap::new();
    args.insert("includeGroup".to_string(), "true".to_string());
    args.insert("includeSecond".to_string(), "false".to_string());
    let changes = collect(&workspace, yaml, &args);
    assert_eq!(changes.len(), 1);
    assert!(matches!(&changes[0], FileChange::Create { path, .. } if path == "lib/one.dart"));

    args.insert("includeGroup".to_string(), "false".to_string());
    args.insert("includeSecond".to_string(), "true".to_string());
    assert!(collect(&workspace, yaml, &args).is_empty());
}

#[test]
fn if_step_with_recipe_refs_expands_and_gates() {
    let workspace = temp_workspace("if_recipe_group");
    let recipes = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes).unwrap();
    std::fs::write(
        recipes.join("child_a.yaml"),
        r#"
id: child_a
steps:
  - create:
      path: "lib/a.dart"
      template: "class A {}\n"
"#,
    )
    .unwrap();
    std::fs::write(
        recipes.join("child_b.yaml"),
        r#"
id: child_b
steps:
  - create:
      path: "lib/b.dart"
      template: "class B {}\n"
"#,
    )
    .unwrap();
    std::fs::write(
        recipes.join("parent_if.yaml"),
        r#"
id: parent_if
args:
  - name: includeChildren
    defaultsTo: "false"
steps:
  - if:
      if: includeChildren
      steps:
        - recipe: child_a
        - recipe: child_b
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();
    let (recipe, path) = registry.load_recipe_ast("parent_if").unwrap();

    let mut args = BTreeMap::new();
    args.insert("includeChildren".to_string(), "false".to_string());
    let skipped = collect_recipe_changes(&registry, &recipe, Some(path.as_path()), &args)
        .unwrap()
        .changes;
    assert!(skipped.is_empty());

    args.insert("includeChildren".to_string(), "true".to_string());
    let applied = collect_recipe_changes(&registry, &recipe, Some(path.as_path()), &args)
        .unwrap()
        .changes;
    assert_eq!(applied.len(), 2);
}
