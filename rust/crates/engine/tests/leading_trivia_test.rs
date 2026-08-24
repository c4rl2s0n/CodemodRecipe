use codemod_recipe_engine::engine::QueryContext;
use codemod_recipe_engine::ops::{EditOp, QuerySpec, RemoveOp};

mod common;

const SOURCE_WITH_DOC_FIELD: &str =
    "class Settings {\n  /// Count of items.\n  final int count = 0;\n  final int other = 1;\n}\n";

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
fn remove_with_leading_trivia_strips_doc_comment() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let ctx = QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };
    let ops = vec![EditOp::Remove(RemoveOp {
        query: QuerySpec::single(FIELD_QUERY),
        capture: "member".to_string(),
        include_leading_trivia: true,
    })];
    let out = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &ops, SOURCE_WITH_DOC_FIELD)
            .unwrap()
    });
    assert!(!out.contains("Count of items"));
    assert!(!out.contains("count"));
    assert!(out.contains("other"));
}

#[test]
fn remove_without_leading_trivia_keeps_doc_comment() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
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
            .apply_edit_ops_sequential(&ctx, &ops, SOURCE_WITH_DOC_FIELD)
            .unwrap()
    });
    assert!(out.contains("Count of items"));
    assert!(!out.contains("count = 0"));
}
