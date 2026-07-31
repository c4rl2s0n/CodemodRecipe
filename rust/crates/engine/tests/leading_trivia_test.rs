use codemod_recipe_engine::engine::QueryContext;
use codemod_recipe_yaml::model::{EditOp, EditStep, Recipe, RemoveOp, Step};
use codemod_recipe_yaml::QuerySpec;

use std::collections::BTreeMap;

mod common;

const SOURCE_WITH_DOC_FIELD: &str =
    "class Settings {\n  /// Count of items.\n  final int count = 0;\n  final int other = 1;\n}\n";

#[test]
fn remove_with_leading_trivia_strips_doc_comment() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let ctx = QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };

    let recipe = Recipe {
        id: "remove_doc_field".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Edit(EditStep {
            path: "test.dart".to_string(),
            ops: vec![EditOp::Remove(RemoveOp {
                query: QuerySpec::single(
                    r#"(class_definition
  name: (identifier) @className
  body: (class_body
    (declaration
      (initialized_identifier_list
        (initialized_identifier
          (identifier) @fieldName))) @member)
  (#eq? @className "Settings")
  (#eq? @fieldName "count"))"#,
                ),
                capture: "member".to_string(),
                include_leading_trivia: true,
            })],
            ..Default::default()
        })],
        post_execution: vec![],
        explorer_menu: None,
    };

    let out = common::with_engine("dart", |engine| {
        engine
            .apply_recipe_to_source(&ctx, &recipe, "test.dart", SOURCE_WITH_DOC_FIELD)
            .unwrap()
            .modified
    });

    assert!(!out.contains("/// Count"));
    assert!(!out.contains("final int count"));
    assert!(out.contains("final int other"));
}

const SOURCE_WITH_LINE_COMMENT_FIELD: &str =
    "class Settings {\n  // item count\n  final int count = 0;\n}\n";

#[test]
fn remove_with_leading_trivia_strips_line_comment() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let ctx = QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };

    let recipe = Recipe {
        id: "remove_line_comment_field".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Edit(EditStep {
            path: "test.dart".to_string(),
            ops: vec![EditOp::Remove(RemoveOp {
                query: QuerySpec::single(
                    r#"(class_definition
  name: (identifier) @className
  body: (class_body
    (declaration
      (initialized_identifier_list
        (initialized_identifier
          (identifier) @fieldName))) @member)
  (#eq? @className "Settings")
  (#eq? @fieldName "count"))"#,
                ),
                capture: "member".to_string(),
                include_leading_trivia: true,
            })],
            ..Default::default()
        })],
        post_execution: vec![],
        explorer_menu: None,
    };

    let out = common::with_engine("dart", |engine| {
        engine
            .apply_recipe_to_source(&ctx, &recipe, "test.dart", SOURCE_WITH_LINE_COMMENT_FIELD)
            .unwrap()
            .modified
    });

    assert!(!out.contains("// item count"));
    assert!(!out.contains("final int count"));
}
