use codemod_recipe_yaml::model::Recipe;
use codemod_recipe_yaml::validate::validate_recipe;

#[test]
fn parses_rust_oracle_recipe_fixture() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../test/fixtures/rust_oracle/insert_log_line.recipe.yaml");

    let text = std::fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("failed to read fixture {fixture_path:?}: {e}"));

    let recipe: Recipe =
        serde_yaml::from_str(&text).unwrap_or_else(|e| panic!("yaml parse failed: {e}"));

    validate_recipe(&recipe).unwrap();

    assert_eq!(recipe.id, "insert_log_line");
    assert_eq!(recipe.steps.len(), 1);
}
