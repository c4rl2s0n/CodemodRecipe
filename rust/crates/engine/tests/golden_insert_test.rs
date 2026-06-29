use codemod_recipe_engine::engine::{parse_recipe_yaml, Engine, QueryContext};
use pretty_assertions::assert_eq;

fn test_ctx<'a>(codemod_root: &'a std::path::Path, recipe_file: Option<&'a std::path::Path>) -> QueryContext<'a> {
    QueryContext {
        recipe_file,
        codemod_root,
    }
}

#[test]
fn golden_insert_log_line() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");

    let recipe_path = repo_root.join("test/fixtures/rust_oracle/insert_log_line.recipe.yaml");
    let after_path =
        repo_root.join("test/fixtures/rust_oracle/settings.after.insert_log_line.dart");
    let before_path = repo_root.join("test/fixtures/ast_paths/settings.dart");

    let recipe_yaml = std::fs::read_to_string(&recipe_path).unwrap();
    let before = std::fs::read_to_string(&before_path).unwrap();
    let expected = std::fs::read_to_string(&after_path).unwrap();

    let recipe = parse_recipe_yaml(&recipe_yaml).unwrap();
    let mut engine = Engine::new_dart().unwrap();
    let codemod = repo_root.join(".codemod");
    let ctx = test_ctx(&codemod, Some(&recipe_path));

    let out = engine
        .apply_recipe_to_source(&ctx, &recipe, "{{file}}", &before)
        .unwrap()
        .modified;

    assert_eq!(out, expected);
}

#[test]
fn golden_insert_log_line_via_query_file() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");

    let recipe_path = repo_root.join("test/fixtures/rust_oracle/insert_log_line_scm.recipe.yaml");
    let after_path =
        repo_root.join("test/fixtures/rust_oracle/settings.after.insert_log_line.dart");
    let before_path = repo_root.join("test/fixtures/ast_paths/settings.dart");

    let recipe_yaml = std::fs::read_to_string(&recipe_path).unwrap();
    let before = std::fs::read_to_string(&before_path).unwrap();
    let expected = std::fs::read_to_string(&after_path).unwrap();

    let recipe = parse_recipe_yaml(&recipe_yaml).unwrap();
    let mut engine = Engine::new_dart().unwrap();
    let codemod = repo_root.join(".codemod");
    let ctx = test_ctx(&codemod, Some(&recipe_path));

    let out = engine
        .apply_recipe_to_source(&ctx, &recipe, "{{file}}", &before)
        .unwrap()
        .modified;

    assert_eq!(out, expected);
}
