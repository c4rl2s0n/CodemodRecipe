use codemod_recipe_engine::engine::QueryContext;
use codemod_recipe_engine::ops::{EditOp, InsertAnchor, InsertOp, QuerySpec};

mod common;

const INSERT_QUERY: &str = r#"(class_definition
  name: (identifier) @className
  body: (class_body
    (method_signature
      (function_signature
        name: (identifier) @methodName))
    (function_body
      (block) @body))
  (#eq? @className "Settings")
  (#eq? @methodName "update"))"#;

fn insert_log_ops(query: QuerySpec) -> Vec<EditOp> {
    vec![EditOp::Insert(InsertOp {
        query,
        capture: "body".to_string(),
        anchor: InsertAnchor::End,
        text: "    print('codemod');\n".to_string(),
    })]
}

fn test_ctx<'a>(
    codemod_root: &'a std::path::Path,
    recipe_file: Option<&'a std::path::Path>,
) -> QueryContext<'a> {
    QueryContext {
        recipe_file,
        codemod_root,
    }
}

#[test]
fn golden_insert_log_line() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let after_path =
        repo_root.join("test/fixtures/rust_oracle/settings.after.insert_log_line.dart");
    let before_path = repo_root.join("test/fixtures/ast_paths/settings.dart");
    let before = std::fs::read_to_string(&before_path).unwrap();
    let expected = std::fs::read_to_string(&after_path).unwrap();
    let codemod = repo_root.join(".codemod");
    let ctx = test_ctx(&codemod, None);
    let ops = insert_log_ops(QuerySpec::single(INSERT_QUERY));

    let out = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &ops, &before)
            .unwrap()
    });
    assert_eq!(out, expected);
}

#[test]
fn golden_insert_log_line_via_query_file() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let recipe_path = repo_root.join("test/fixtures/rust_oracle/insert_log_line_scm.recipe.yaml");
    let after_path =
        repo_root.join("test/fixtures/rust_oracle/settings.after.insert_log_line.dart");
    let before_path = repo_root.join("test/fixtures/ast_paths/settings.dart");
    let before = std::fs::read_to_string(&before_path).unwrap();
    let expected = std::fs::read_to_string(&after_path).unwrap();
    let codemod = repo_root.join(".codemod");
    let ctx = test_ctx(&codemod, Some(&recipe_path));
    let ops = insert_log_ops(QuerySpec::single("settings_update_body.scm"));

    let out = common::with_engine("dart", |engine| {
        engine
            .apply_edit_ops_sequential(&ctx, &ops, &before)
            .unwrap()
    });
    assert_eq!(out, expected);
}
