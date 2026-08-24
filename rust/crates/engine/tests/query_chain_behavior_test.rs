use codemod_recipe_engine::engine::{EngineError, QueryContext};
use codemod_recipe_engine::ops::{EditOp, InsertAnchor, InsertOp, QuerySpec};

mod common;

const CLASS_STEP: &str = "(class_definition) @class";

const METHOD_TARGET_BODY: &str = r#"(class_body
  (method_signature
    (function_signature name: (identifier) @methodName))
  (function_body (block) @body)
  (#eq? @methodName "target"))"#;

const METHOD_UPDATE_BODY: &str = r#"(class_body
  (method_signature
    (function_signature name: (identifier) @methodName))
  (function_body (block) @body)
  (#eq? @methodName "update"))"#;

const TWO_METHOD_BODIES: &str = r#"(class_definition
  body: (class_body
    (method_signature
      (function_signature name: (identifier) @methodName))
    (function_body (block) @body)))"#;

fn ctx() -> QueryContext<'static> {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let codemod_static: &'static std::path::Path = Box::leak(codemod.into_boxed_path());
    QueryContext {
        recipe_file: None,
        codemod_root: codemod_static,
    }
}

fn insert_ops(query: QuerySpec, capture: &str) -> Vec<EditOp> {
    vec![EditOp::Insert(InsertOp {
        query,
        capture: capture.to_string(),
        anchor: InsertAnchor::End,
        text: "/* marker */\n".to_string(),
    })]
}

#[test]
fn chain_fan_out_only_applies_where_later_step_matches() {
    let ctx = ctx();
    let source = "class A {\n  void target() {}\n}\nclass B {\n  void other() {}\n}\n";
    let query = QuerySpec::Chain(vec![CLASS_STEP.to_string(), METHOD_TARGET_BODY.to_string()]);
    let ops = insert_ops(query, "body");
    let out = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &ops, source)
            .unwrap()
    });
    assert!(out.contains("class A"));
    assert!(out.contains("/* marker */"));
    let b_idx = out.find("class B").unwrap();
    assert!(!out[b_idx..].contains("/* marker */"));
}

#[test]
fn chain_no_match_on_later_step_errors_for_insert() {
    let ctx = ctx();
    let source = "class A {\n  void other() {}\n}\n";
    let query = QuerySpec::Chain(vec![CLASS_STEP.to_string(), METHOD_UPDATE_BODY.to_string()]);
    let ops = insert_ops(query, "body");
    let err = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &ops, source)
            .unwrap_err()
    });
    assert!(matches!(err, EngineError::NoMatch { .. }));
}

#[test]
fn ambiguous_final_capture_errors() {
    let ctx = ctx();
    let source = "class A {\n  void a() {}\n  void b() {}\n}\n";
    let ops = insert_ops(QuerySpec::single(TWO_METHOD_BODIES), "body");
    let err = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &ops, source)
            .unwrap_err()
    });
    assert!(matches!(err, EngineError::MultipleMatches { .. }));
}

#[test]
fn chain_then_ambiguous_final_capture_errors() {
    let ctx = ctx();
    let source = "class A {\n  void a() {}\n  void b() {}\n}\n";
    let query = QuerySpec::Chain(vec![
        CLASS_STEP.to_string(),
        r#"(class_body
  (method_signature (function_signature name: (identifier) @methodName))
  (function_body (block) @body))"#
            .to_string(),
    ]);
    let ops = insert_ops(query, "body");
    let err = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &ops, source)
            .unwrap_err()
    });
    assert!(matches!(err, EngineError::MultipleMatches { .. }));
}
