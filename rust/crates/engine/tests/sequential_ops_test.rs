use codemod_recipe_engine::engine::{EngineError, QueryContext};
use codemod_recipe_engine::ops::{EditOp, InsertAnchor, InsertOp, QuerySpec};

mod common;

fn sequential_ops() -> Vec<EditOp> {
    vec![
        EditOp::Insert(InsertOp {
            query: QuerySpec::single(
                r#"(class_definition
  name: (identifier) @name
  body: (class_body) @body
  (#eq? @name "Foo"))"#,
            ),
            capture: "body".to_string(),
            anchor: InsertAnchor::End,
            text: "  void bar() {}\n".to_string(),
        }),
        EditOp::Insert(InsertOp {
            query: QuerySpec::single(
                r#"(class_definition
  name: (identifier) @className
  body: (class_body
    (method_signature
      (function_signature
        name: (identifier) @methodName)) @member)
  (#eq? @className "Foo")
  (#eq? @methodName "bar"))"#,
            ),
            capture: "member".to_string(),
            anchor: InsertAnchor::End,
            text: "\n  Foo();".to_string(),
        }),
    ]
}

#[test]
fn sequential_ops_second_insert_sees_first() {
    let source = "class Foo {\n}\n";
    let codemod = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.codemod");
    let ctx = QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };
    let ops = sequential_ops();
    let out = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &ops, source)
            .unwrap()
    });
    assert!(out.contains("void bar()"), "first insert missing: {out}");
    assert!(out.contains("Foo();"), "second insert missing: {out}");
}

#[test]
fn non_sequential_collect_misses_dependent_second_op() {
    let source = "class Foo {\n}\n";
    let codemod = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../.codemod");
    let ctx = QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };
    let ops = sequential_ops();
    let err = common::with_engine("dart", |engine| {
        engine
            .collect_patches_for_ops(&ctx, &ops, source)
            .unwrap_err()
    });
    assert!(
        matches!(err, EngineError::NoMatch { .. }),
        "second insert should fail to match original source: {err}"
    );
}
