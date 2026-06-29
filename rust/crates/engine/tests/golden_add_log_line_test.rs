use codemod_recipe_engine::engine::{parse_recipe_yaml, Engine, QueryContext};
use codemod_recipe_host::registry::render_recipe_templates;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

#[test]
fn golden_add_log_line_parameterized() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");

    let recipe_path = repo_root.join("test/fixtures/rust_oracle/add_log_line.recipe.yaml");
    let after_path =
        repo_root.join("test/fixtures/rust_oracle/settings.after.insert_log_line.dart");
    let before_path = repo_root.join("test/fixtures/ast_paths/settings.dart");

    let recipe_yaml = std::fs::read_to_string(&recipe_path).unwrap();
    let before = std::fs::read_to_string(&before_path).unwrap();
    let expected = std::fs::read_to_string(&after_path).unwrap();

    let recipe = parse_recipe_yaml(&recipe_yaml).unwrap();
    let mut args = BTreeMap::new();
    args.insert(
        "file".to_string(),
        "test/fixtures/ast_paths/settings.dart".to_string(),
    );
    args.insert("className".to_string(), "Settings".to_string());
    args.insert("methodName".to_string(), "update".to_string());

    let rendered = render_recipe_templates(&recipe, &args, &BTreeMap::new());
    let mut engine = Engine::new_dart().unwrap();
    let codemod = repo_root.join(".codemod");
    let ctx = QueryContext {
        recipe_file: Some(recipe_path.as_path()),
        codemod_root: &codemod,
    };

    let out = engine
        .apply_recipe_to_source(&ctx, &rendered, args["file"].as_str(), &before)
        .unwrap()
        .modified;

    assert_eq!(out, expected);
}

#[test]
fn matches_insert_log_line_for_settings_update() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let insert_path = repo_root.join("test/fixtures/rust_oracle/insert_log_line.recipe.yaml");
    let add_path = repo_root.join("test/fixtures/rust_oracle/add_log_line.recipe.yaml");
    let before_path = repo_root.join("test/fixtures/ast_paths/settings.dart");

    let before = std::fs::read_to_string(&before_path).unwrap();
    let insert = parse_recipe_yaml(&std::fs::read_to_string(&insert_path).unwrap()).unwrap();
    let add = parse_recipe_yaml(&std::fs::read_to_string(&add_path).unwrap()).unwrap();

    let mut args = BTreeMap::new();
    args.insert(
        "file".to_string(),
        "test/fixtures/ast_paths/settings.dart".to_string(),
    );
    args.insert("className".to_string(), "Settings".to_string());
    args.insert("methodName".to_string(), "update".to_string());

    let codemod = repo_root.join(".codemod");
    let mut engine = Engine::new_dart().unwrap();

    let insert_out = engine
        .apply_recipe_to_source(
            &QueryContext {
                recipe_file: Some(insert_path.as_path()),
                codemod_root: &codemod,
            },
            &render_recipe_templates(&insert, &args, &BTreeMap::new()),
            args["file"].as_str(),
            &before,
        )
        .unwrap()
        .modified;

    let add_out = engine
        .apply_recipe_to_source(
            &QueryContext {
                recipe_file: Some(add_path.as_path()),
                codemod_root: &codemod,
            },
            &render_recipe_templates(&add, &args, &BTreeMap::new()),
            args["file"].as_str(),
            &before,
        )
        .unwrap()
        .modified;

    assert_eq!(insert_out, add_out);
}
