use codemod_recipe_engine::engine::QueryContext;
use codemod_recipe_engine::ops::{EditOp, QuerySpec, RemoveOp};
use pretty_assertions::assert_eq;

mod common;

const FIELD_QUERY: &str = r#"(class_definition
  name: (identifier) @className
  body: (class_body
    (declaration
      (initialized_identifier_list
        (initialized_identifier
          (identifier) @fieldName))) @member)
  (#eq? @className "Settings")
  (#eq? @fieldName "count"))"#;

#[test]
fn golden_remove_count_field() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let before_path = repo_root.join("test/fixtures/ast_paths/settings.dart");
    let after_path = repo_root.join("test/fixtures/rust_oracle/settings.after.remove_count.dart");
    let before = std::fs::read_to_string(&before_path).unwrap();
    let expected = std::fs::read_to_string(&after_path).unwrap();
    let codemod = repo_root.join(".codemod");
    let ctx = QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };
    let ops = vec![EditOp::Remove(RemoveOp {
        query: QuerySpec::single(FIELD_QUERY),
        capture: "member".to_string(),
        include_leading_trivia: false,
    })];
    let out = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &ops, &before)
            .unwrap()
    });
    assert_eq!(out, expected);
}
