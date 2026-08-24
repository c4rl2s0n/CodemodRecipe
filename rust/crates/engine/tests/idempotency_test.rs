use codemod_recipe_engine::engine::QueryContext;
use codemod_recipe_engine::ops::{EditOp, InsertAnchor, InsertOp, QuerySpec, RemoveOp, ReplaceOp};

mod common;

const SETTINGS: &str = "class Settings {\n  final int count = 0;\n}\n";
const SETTINGS_WITHOUT_COUNT: &str = "class Settings {\n}\n";

const FIELD_QUERY: &str = r#"(class_definition
  name: (identifier) @className
  body: (class_body
    (declaration
      (initialized_identifier_list
        (initialized_identifier
          (identifier) @fieldName))) @member)
  (#eq? @className "Settings")
  (#eq? @fieldName "count"))"#;

fn inline_ctx<'a>(codemod_root: &'a std::path::Path) -> QueryContext<'a> {
    QueryContext {
        recipe_file: None,
        codemod_root,
    }
}

fn remove_ops() -> Vec<EditOp> {
    vec![EditOp::Remove(RemoveOp {
        query: QuerySpec::single(FIELD_QUERY),
        capture: "member".to_string(),
        include_leading_trivia: false,
    })]
}

fn replace_ops(text: &str) -> Vec<EditOp> {
    vec![EditOp::Replace(ReplaceOp {
        query: QuerySpec::single(FIELD_QUERY),
        capture: "member".to_string(),
        text: text.to_string(),
        include_leading_trivia: false,
    })]
}

#[test]
fn remove_deletes_field_declaration() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let ctx = inline_ctx(&codemod);
    let out = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &remove_ops(), SETTINGS)
            .unwrap()
    });
    assert_eq!(out, SETTINGS_WITHOUT_COUNT);
}

#[test]
fn remove_no_ops_when_field_absent() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let ctx = inline_ctx(&codemod);
    let out = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &remove_ops(), SETTINGS_WITHOUT_COUNT)
            .unwrap()
    });
    assert_eq!(out, SETTINGS_WITHOUT_COUNT);
}

#[test]
fn replace_no_ops_when_whitespace_normalized_text_matches() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let ctx = inline_ctx(&codemod);
    let out = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &replace_ops("final int count = 0;"), SETTINGS)
            .unwrap()
    });
    assert_eq!(out, SETTINGS);
}

#[test]
fn insert_anchor_start_is_available() {
    let _ = InsertAnchor::Start;
}
