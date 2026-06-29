use codemod_recipe_engine::engine::{parse_recipe_yaml, Engine, QueryContext};
use pretty_assertions::assert_eq;

#[test]
fn golden_remove_count_field() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");

    let recipe_path = repo_root.join("test/fixtures/rust_oracle/remove_count_field.recipe.yaml");
    let before_path = repo_root.join("test/fixtures/ast_paths/settings.dart");
    let after_path = repo_root.join("test/fixtures/rust_oracle/settings.after.remove_count.dart");

    let recipe_yaml = std::fs::read_to_string(&recipe_path).unwrap();
    let before = std::fs::read_to_string(&before_path).unwrap();
    let expected = std::fs::read_to_string(&after_path).unwrap();

    let recipe = parse_recipe_yaml(&recipe_yaml).unwrap();
    let mut engine = Engine::new_dart().unwrap();
    let codemod = repo_root.join(".codemod");
    let ctx = QueryContext {
        recipe_file: Some(recipe_path.as_path()),
        codemod_root: &codemod,
    };
    let out = engine
        .apply_recipe_to_source(&ctx, &recipe, "{{file}}", &before)
        .unwrap()
        .modified;

    assert_eq!(out, expected);
}
