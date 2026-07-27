//! Edit-step `when` / `whenNot` / `let` integration.

use codemod_recipe_core::file_change::FileChange;
use codemod_recipe_core::patch::apply_patches;
use codemod_recipe_engine::engine::parse_recipe_yaml;
use codemod_recipe_host::registry::RecipeRegistry;
use codemod_recipe_host::runner::collect_recipe_changes;
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

fn collect_patched(
    workspace: &std::path::Path,
    yaml: &str,
    args: &BTreeMap<String, String>,
) -> (Vec<codemod_recipe_core::file_change::FileChange>, Option<String>) {
    let recipe = parse_recipe_yaml(yaml).expect("parse");
    let codemod = workspace.join(".codemod");
    std::fs::create_dir_all(&codemod).unwrap();
    let registry = RecipeRegistry::new(workspace.to_path_buf(), codemod);
    let collected = collect_recipe_changes(&registry, &recipe, None, args).expect("collect");
    let out = collected
        .changes
        .first()
        .map(|c| {
            let FileChange::Patch { source, patches, .. } = c else {
                panic!("expected patch");
            };
            apply_patches(source, patches).expect("apply")
        });
    (collected.changes, out)
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
    let (changes, _) = collect_patched(&workspace, yaml, &BTreeMap::new());
    assert!(changes.is_empty(), "whenNot should skip edit");
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn when_not_allows_edit_when_pattern_absent() {
    let workspace = temp_workspace("when_not_run");
    std::fs::write(
        workspace.join("lib/a.dart"),
        "class Plain {\n}\n",
    )
    .unwrap();
    let yaml = r#"
id: t
steps:
  - edit:
      path: lib/a.dart
      language: dart
      whenNot:
        - |
          (class_definition
            name: (identifier) @n
            (#eq? @n "HasMarker"))
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: "\n// ok\n"
"#;
    let (changes, out) = collect_patched(&workspace, yaml, &BTreeMap::new());
    assert_eq!(changes.len(), 1);
    assert!(out.unwrap().contains("// ok"));
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn when_skips_edit_when_required_pattern_missing() {
    let workspace = temp_workspace("when_skip");
    std::fs::write(workspace.join("lib/a.dart"), "class Other {\n}\n").unwrap();
    let yaml = r#"
id: t
steps:
  - edit:
      path: lib/a.dart
      language: dart
      when:
        - |
          (class_definition
            name: (identifier) @n
            (#eq? @n "Target"))
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: "\n// no\n"
"#;
    let (changes, _) = collect_patched(&workspace, yaml, &BTreeMap::new());
    assert!(changes.is_empty());
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn when_applies_edit_when_pattern_present() {
    let workspace = temp_workspace("when_run");
    std::fs::write(workspace.join("lib/a.dart"), "class Target {\n}\n").unwrap();
    let yaml = r#"
id: t
steps:
  - edit:
      path: lib/a.dart
      language: dart
      when:
        - |
          (class_definition
            name: (identifier) @n
            (#eq? @n "Target"))
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: "\n// yes\n"
"#;
    let (changes, out) = collect_patched(&workspace, yaml, &BTreeMap::new());
    assert_eq!(changes.len(), 1);
    assert!(out.unwrap().contains("// yes"));
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn when_list_requires_all_guards() {
    let workspace = temp_workspace("when_all");
    std::fs::write(workspace.join("lib/a.dart"), "class Target {\n}\n").unwrap();
    let yaml = r#"
id: t
steps:
  - edit:
      path: lib/a.dart
      language: dart
      when:
        - |
          (class_definition
            name: (identifier) @n
            (#eq? @n "Target"))
        - |
          (class_definition
            name: (identifier) @n
            (#eq? @n "Other"))
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: "\n// no\n"
"#;
    let (changes, _) = collect_patched(&workspace, yaml, &BTreeMap::new());
    assert!(changes.is_empty());
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn when_and_when_not_both_must_pass() {
    let workspace = temp_workspace("when_combo");
    std::fs::write(
        workspace.join("lib/a.dart"),
        "class Target {\n  final int ok = 0;\n}\n",
    )
    .unwrap();
    let yaml = r#"
id: t
steps:
  - edit:
      path: lib/a.dart
      language: dart
      when:
        - |
          (class_definition
            name: (identifier) @n
            (#eq? @n "Target"))
      whenNot:
        - |
          (class_definition
            body: (class_body
              (declaration
                (initialized_identifier_list
                  (initialized_identifier
                    (identifier) @f)))
              (#eq? @f "blocked")))
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: "\n// combo\n"
"#;
    let (changes, out) = collect_patched(&workspace, yaml, &BTreeMap::new());
    assert_eq!(changes.len(), 1);
    assert!(out.unwrap().contains("// combo"));
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn when_with_recipe_arg_in_deferred_op_render() {
    let workspace = temp_workspace("when_defer_render");
    std::fs::write(workspace.join("lib/a.dart"), "class Target {\n}\n").unwrap();
    let yaml = r#"
id: t
args:
  - name: marker
    required: true
steps:
  - edit:
      path: lib/a.dart
      language: dart
      when:
        - |
          (class_definition
            name: (identifier) @n
            (#eq? @n "Target"))
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: "\n// {{ marker }}\n"
"#;
    let args = BTreeMap::from([("marker".to_string(), "from-arg".to_string())]);
    let (changes, out) = collect_patched(&workspace, yaml, &args);
    assert_eq!(changes.len(), 1);
    assert!(out.unwrap().contains("// from-arg"));
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
    let (changes, out) = collect_patched(&workspace, yaml, &BTreeMap::new());
    assert_eq!(changes.len(), 1);
    assert!(out.unwrap().contains("// let=Foo"));
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn let_as_with_numeric_filters_in_recipe() {
    let workspace = temp_workspace("let_as");
    std::fs::write(workspace.join("lib/a.dart"), "class N {\n}\n").unwrap();
    let yaml = r#"
id: t
args:
  - name: version
    required: false
    defaultsTo: "3"
steps:
  - edit:
      path: lib/a.dart
      language: dart
      let:
        - name: raw
          as: "{{ version }}"
        - name: next
          as: "{{ raw | int | add(1) | string }}"
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: "\n// v={{ next }}\n"
"#;
    let (changes, out) = collect_patched(&workspace, yaml, &BTreeMap::new());
    assert_eq!(changes.len(), 1);
    assert!(out.unwrap().contains("// v=4"));
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn let_ordered_bindings_second_uses_first() {
    let workspace = temp_workspace("let_ordered");
    std::fs::write(workspace.join("lib/a.dart"), "class X {\n}\n").unwrap();
    let yaml = r#"
id: t
steps:
  - edit:
      path: lib/a.dart
      language: dart
      let:
        - name: label
          query: |
            (class_definition
              name: (identifier) @n)
          capture: n
          extract: text
        - name: greeting
          as: "hi-{{ label }}"
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: "\n// {{ greeting }}\n"
"#;
    let (changes, out) = collect_patched(&workspace, yaml, &BTreeMap::new());
    assert_eq!(changes.len(), 1);
    assert!(out.unwrap().contains("// hi-X"));
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn let_rereads_source_before_each_op() {
    let workspace = temp_workspace("let_per_op");
    std::fs::write(workspace.join("lib/a.dart"), "class Grow {\n}\n").unwrap();
    let yaml = r#"
id: t
steps:
  - edit:
      path: lib/a.dart
      language: dart
      let:
        - name: hasMarker
          query: |
            (comment) @c
            (#eq? @c "// MARKER")
          capture: c
          extract: exists
      ops:
        - insert:
            query: |
              (class_definition
                body: (class_body) @body)
            capture: body
            anchor: end
            text: "\n  // MARKER\n"
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: "\n// exists={{ hasMarker }}\n"
"#;
    let (changes, out) = collect_patched(&workspace, yaml, &BTreeMap::new());
    assert_eq!(changes.len(), 1);
    let text = out.unwrap();
    assert!(text.contains("// MARKER"));
    assert!(
        text.contains("// exists=true"),
        "second op should see marker from first op; got {text}"
    );
    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn let_chained_query_extracts_field_in_class() {
    let workspace = repo_root_when_let();
    let codemod = workspace.join(".codemod");
    std::fs::create_dir_all(&codemod).ok();
    std::fs::create_dir_all(workspace.join("lib")).ok();
    std::fs::write(
        workspace.join("lib/a.dart"),
        "class Demo {\n  void beta() {}\n}\n",
    )
    .unwrap();
    let yaml = r#"
id: t
steps:
  - edit:
      path: lib/a.dart
      language: dart
      let:
        - name: field
          query:
            - dart_queries.class_named
            - dart_queries.method_body
          capture: methodName
          extract: text
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: "\n// field={{ field }}\n"
"#;
    let recipe = parse_recipe_yaml(yaml).expect("parse");
    let mut registry = RecipeRegistry::new(workspace.clone(), codemod);
    registry.reload();
    let args = BTreeMap::from([
        ("className".to_string(), "Demo".to_string()),
        ("methodName".to_string(), "beta".to_string()),
    ]);
    let collected = collect_recipe_changes(&registry, &recipe, None, &args).expect("collect");
    assert_eq!(collected.changes.len(), 1);
    let out = collected
        .changes
        .first()
        .and_then(|c| c.modified_content().ok().flatten())
        .expect("modified");
    assert!(
        out.contains("// field=beta"),
        "expected chained let to resolve method name, got: {out}"
    );
    let _ = std::fs::remove_file(workspace.join("lib/a.dart"));
}

fn repo_root_when_let() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn let_local_in_query_predicate_via_template() {
    let workspace = temp_workspace("let_in_query");
    std::fs::write(
        workspace.join("lib/a.dart"),
        "class Pick {\n  void alpha() {}\n  void beta() {}\n}\n",
    )
    .unwrap();
    let yaml = r#"
id: t
args:
  - name: method
    required: true
steps:
  - edit:
      path: lib/a.dart
      language: dart
      let:
        - name: target
          as: "{{ method }}"
      ops:
        - insert:
            query: |
              (class_definition
                body: (class_body
                  (method_signature
                    (function_signature
                      name: (identifier) @m))
                  (function_body
                    (block) @body))
                (#eq? @m "{{ target }}"))
            capture: body
            anchor: end
            text: " // tagged"
"#;
    let args = BTreeMap::from([("method".to_string(), "beta".to_string())]);
    let (changes, out) = collect_patched(&workspace, yaml, &args);
    assert_eq!(changes.len(), 1);
    assert!(out.unwrap().contains("// tagged"));
    let _ = std::fs::remove_dir_all(workspace);
}
