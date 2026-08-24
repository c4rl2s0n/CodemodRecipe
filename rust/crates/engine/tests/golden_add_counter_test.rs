use codemod_recipe_engine::engine::QueryContext;
use codemod_recipe_engine::ops::{EditOp, InsertAnchor, InsertOp, QuerySpec};
use pretty_assertions::assert_eq;

mod common;

#[test]
fn golden_add_counter_field() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let before_path = repo_root.join("test/fixtures/ast_paths/settings.dart");
    let before = std::fs::read_to_string(&before_path).unwrap();
    let codemod = repo_root.join(".codemod");
    let ctx = QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };
    let ops = vec![EditOp::Insert(InsertOp {
        query: QuerySpec::single(
            r#"(class_definition
  name: (identifier) @className
  body: (class_body) @body
  (#eq? @className "Settings"))"#,
        ),
        capture: "body".to_string(),
        anchor: InsertAnchor::Start,
        text: "  final int counter;\n".to_string(),
    })];
    let out = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &ops, &before)
            .unwrap()
    });
    assert!(
        out.contains("final int counter;"),
        "expected counter field in output:\n{out}"
    );
    assert!(
        out.find("final int counter;").unwrap() < out.find("void update").unwrap(),
        "counter field should appear before update()"
    );
}

#[test]
fn golden_add_log_line_parameterized() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let after_path =
        repo_root.join("test/fixtures/rust_oracle/settings.after.insert_log_line.dart");
    let before_path = repo_root.join("test/fixtures/ast_paths/settings.dart");
    let before = std::fs::read_to_string(&before_path).unwrap();
    let expected = std::fs::read_to_string(&after_path).unwrap();
    let codemod = repo_root.join(".codemod");
    let ctx = QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };
    let ops = vec![EditOp::Insert(InsertOp {
        query: QuerySpec::single(
            r#"(class_definition
  name: (identifier) @className
  body: (class_body
    (method_signature
      (function_signature
        name: (identifier) @methodName))
    (function_body
      (block) @body))
  (#eq? @className "Settings")
  (#eq? @methodName "update"))"#,
        ),
        capture: "body".to_string(),
        anchor: InsertAnchor::End,
        text: "    print('codemod');\n".to_string(),
    })];
    let out = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &ops, &before)
            .unwrap()
    });
    assert_eq!(out, expected);
}
