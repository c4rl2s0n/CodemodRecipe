use codemod_recipe_engine::engine::{parse_recipe_yaml, QueryContext};
use codemod_recipe_yaml::model::Step;
use std::path::Path;

mod common;

/// Two dependent inserts in one edit: second op matches text only present after the first.
#[test]
fn sequential_ops_second_insert_sees_first() {
    let source = "class Foo {\n}\n";
    let recipe_yaml = r#"
id: sequential_ops
steps:
  - edit:
      path: "lib/foo.dart"
      ops:
        - insert:
            query: |
              (class_definition
                name: (identifier) @name
                body: (class_body) @body
                (#eq? @name "Foo"))
            capture: body
            anchor: end
            text: "  void bar() {}\n"
        - insert:
            query: |
              (class_definition
                name: (identifier) @className
                body: (class_body
                  (method_signature
                    (function_signature
                      name: (identifier) @methodName)) @member)
                (#eq? @className "Foo")
                (#eq? @methodName "bar"))
            capture: member
            anchor: end
            text: "\n  Foo();"
"#;

    let recipe = parse_recipe_yaml(recipe_yaml).unwrap();
    let codemod = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.codemod");
    let ctx = QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };

    let out = common::with_engine("dart", |engine| {
        engine
            .apply_recipe_to_source(&ctx, &recipe, "lib/foo.dart", source)
            .unwrap()
            .modified
    });

    assert!(out.contains("void bar()"), "first insert missing: {out}");
    assert!(out.contains("Foo();"), "second insert missing: {out}");
}

/// Non-sequential collect against original source cannot see the first insert.
#[test]
fn non_sequential_collect_misses_dependent_second_op() {
    let source = "class Foo {\n}\n";
    let recipe_yaml = r#"
id: sequential_ops
steps:
  - edit:
      path: "lib/foo.dart"
      ops:
        - insert:
            query: |
              (class_definition
                name: (identifier) @name
                body: (class_body) @body
                (#eq? @name "Foo"))
            capture: body
            anchor: end
            text: "  void bar() {}\n"
        - insert:
            query: |
              (class_definition
                name: (identifier) @className
                body: (class_body
                  (method_signature
                    (function_signature
                      name: (identifier) @methodName)) @member)
                (#eq? @className "Foo")
                (#eq? @methodName "bar"))
            capture: member
            anchor: end
            text: "\n  Foo();"
"#;

    let recipe = parse_recipe_yaml(recipe_yaml).unwrap();
    let codemod = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.codemod");
    let ctx = QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };

    let Step::Edit(edit) = &recipe.steps[0] else {
        panic!("expected edit step");
    };

    let err = common::with_engine("dart", |engine| {
        engine
            .collect_patches_for_edit(&ctx, edit, source)
            .unwrap_err()
    });
    assert!(
        err.to_string().contains("matched no nodes"),
        "expected no-match for dependent op against original source, got: {err}"
    );
}
