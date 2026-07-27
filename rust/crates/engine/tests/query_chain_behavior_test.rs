//! Chained query fan-out, no-match, and ambiguous final capture.

use codemod_recipe_engine::engine::{EngineError, QueryContext};
use codemod_recipe_yaml::model::{EditOp, EditStep, InsertAnchor, InsertOp, Recipe, Step};
use codemod_recipe_yaml::QuerySpec;

use std::collections::BTreeMap;

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

fn ctx() -> (std::path::PathBuf, QueryContext<'static>) {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let codemod_static: &'static std::path::Path =
        Box::leak(codemod.into_boxed_path());
    (
        repo,
        QueryContext {
            recipe_file: None,
            codemod_root: codemod_static,
        },
    )
}

fn insert_chain_recipe(query: QuerySpec, capture: &str) -> Recipe {
    Recipe {
        id: "chain_behavior".to_string(),
        name: None,
        description: None,
        group: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Edit(EditStep {
            path: "test.dart".to_string(),
            ops: vec![EditOp::Insert(InsertOp {
                query,
                capture: capture.to_string(),
                anchor: InsertAnchor::End,
                text: "/* marker */\n".to_string(),
            })],
            ..Default::default()
        })],
        post_execution: vec![],
    }
}

#[test]
fn chain_fan_out_only_applies_where_later_step_matches() {
    let (_repo, ctx) = ctx();
    let source = "class A {\n  void target() {}\n}\nclass B {\n  void other() {}\n}\n";
    let query = QuerySpec::Chain(vec![
        CLASS_STEP.to_string(),
        METHOD_TARGET_BODY.to_string(),
    ]);
    let recipe = insert_chain_recipe(query, "body");

    let out = common::with_engine("dart", |engine| {
        engine
            .apply_recipe_to_source(&ctx, &recipe, "test.dart", source)
            .unwrap()
            .modified
    });

    assert!(out.contains("/* marker */"));
    assert_eq!(out.matches("/* marker */").count(), 1);
    assert!(out.contains("void target()"));
    assert!(out.contains("void other()"));
}

#[test]
fn chain_zero_matches_after_scoped_step_errors() {
    let (_repo, ctx) = ctx();
    let source = "class A {\n  void other() {}\n}\n";
    let query = QuerySpec::Chain(vec![
        CLASS_STEP.to_string(),
        METHOD_TARGET_BODY.to_string(),
    ]);
    let recipe = insert_chain_recipe(query, "body");

    let err = common::with_engine("dart", |engine| {
        match engine.apply_recipe_to_source(&ctx, &recipe, "test.dart", source) {
            Ok(_) => panic!("expected NoMatch"),
            Err(e) => e,
        }
    });

    assert!(matches!(err, EngineError::NoMatch { .. }));
}

#[test]
fn chain_multiple_final_captures_errors() {
    let (_repo, ctx) = ctx();
    let source = "class Foo {\n  void a() {}\n  void b() {}\n}\n";
    let recipe = insert_chain_recipe(QuerySpec::single(TWO_METHOD_BODIES), "body");

    let err = common::with_engine("dart", |engine| {
        match engine.apply_recipe_to_source(&ctx, &recipe, "test.dart", source) {
            Ok(_) => panic!("expected NoMatch"),
            Err(e) => e,
        }
    });

    match err {
        EngineError::MultipleMatches { capture, count } => {
            assert_eq!(capture, "body");
            assert_eq!(count, 2);
        }
        other => panic!("expected MultipleMatches, got {other:?}"),
    }
}

#[test]
fn chain_mixed_inline_and_steps_resolves() {
    let (_repo, ctx) = ctx();
    let source = "class Settings {\n  void update() {}\n}\n";
    let query = QuerySpec::Chain(vec![
        r#"(class_definition
  name: (identifier) @className
  body: (class_body) @classBody
  (#eq? @className "Settings"))"#
            .to_string(),
        METHOD_UPDATE_BODY.to_string(),
    ]);
    let recipe = insert_chain_recipe(query, "body");

    let out = common::with_engine("dart", |engine| {
        engine
            .apply_recipe_to_source(&ctx, &recipe, "test.dart", source)
            .unwrap()
            .modified
    });

    assert!(out.contains("/* marker */"));
}
