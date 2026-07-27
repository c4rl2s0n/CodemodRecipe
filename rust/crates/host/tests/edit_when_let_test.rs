//! Edit-step `when` / `whenNot` / `let` integration.

use codemod_recipe_host::runner::collect_recipe_changes;
use codemod_recipe_host::registry::RecipeRegistry;
use codemod_recipe_engine::engine::parse_recipe_yaml;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn temp_workspace(name: &str) -> std::path::PathBuf {
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("codemod_when_let_{name}_{n}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    dir
}

#[test]
fn when_not_skips_edit_when_forbidden_pattern_matches() {
    let workspace = temp_workspace("when_not_skip");
    let dart = r#"class Settings {
  final int migrated = 0;
}
"#;
    std::fs::write(workspace.join("lib/a.dart"), dart).unwrap();

    let yaml = r#"
dslVersion: 2
id: when_not_test
steps:
  - edit:
      path: lib/a.dart
      language: dart
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
            text: "\n// should not appear\n"
"#;
    let recipe = parse_recipe_yaml(yaml).expect("parse");
    let codemod = workspace.join(".codemod");
    std::fs::create_dir_all(&codemod).unwrap();
    let registry = RecipeRegistry::new(workspace.clone(), codemod);
    let collected = collect_recipe_changes(&registry, &recipe, None, &BTreeMap::new())
        .expect("collect");
    assert!(collected.changes.is_empty(), "whenNot should skip edit");

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn let_exists_drives_insert_text_per_op() {
    let workspace = temp_workspace("let_exists");
    let dart = r#"class Foo {
  Foo() {}
}
"#;
    std::fs::write(workspace.join("lib/foo.dart"), dart).unwrap();

    let yaml = r#"
dslVersion: 2
id: let_test
steps:
  - edit:
      path: lib/foo.dart
      language: dart
      let:
        - name: classLabel
          query: |
            (class_definition
              name: (identifier) @n
              (#eq? @n "Foo"))
          capture: n
          extract: text
      ops:
        - insert:
            query: |
              (class_definition
                name: (identifier) @n
                body: (class_body) @body)
              (#eq? @n "Foo")
            capture: body
            anchor: end
            text: "\n// let={{ classLabel }}\n"
"#;
    let recipe = parse_recipe_yaml(yaml).expect("parse");
    let codemod = workspace.join(".codemod");
    std::fs::create_dir_all(&codemod).unwrap();
    let registry = RecipeRegistry::new(workspace.clone(), codemod);
    let collected = collect_recipe_changes(&registry, &recipe, None, &BTreeMap::new())
        .expect("collect");
    assert_eq!(collected.changes.len(), 1);
    use codemod_recipe_core::file_change::FileChange;
    use codemod_recipe_core::patch::apply_patches;
    let change = &collected.changes[0];
    let FileChange::Patch { source, patches, .. } = change else {
        panic!("expected patch");
    };
    let out = apply_patches(source, patches).expect("apply");
    assert!(out.contains("// let=Foo"), "out={out}");

    let _ = std::fs::remove_dir_all(workspace);
}
