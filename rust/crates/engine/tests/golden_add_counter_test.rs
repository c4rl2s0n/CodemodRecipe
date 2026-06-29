use codemod_recipe_engine::engine::{parse_recipe_yaml, Engine, QueryContext};
use codemod_recipe_host::registry::render_recipe_templates;
use codemod_recipe_host::template::render_string;
use pretty_assertions::assert_eq;
use std::collections::BTreeMap;

#[test]
fn golden_add_counter_field() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");

    let recipe_path = repo_root.join("test/fixtures/rust_oracle/add_counter_field.recipe.yaml");
    let before_path = repo_root.join("test/fixtures/ast_paths/settings.dart");

    let recipe_yaml = std::fs::read_to_string(&recipe_path).unwrap();
    let before = std::fs::read_to_string(&before_path).unwrap();

    let recipe = parse_recipe_yaml(&recipe_yaml).unwrap();
    let mut args = BTreeMap::new();
    args.insert(
        "file".to_string(),
        "test/fixtures/ast_paths/settings.dart".to_string(),
    );
    args.insert("className".to_string(), "Settings".to_string());
    args.insert("field".to_string(), "counter".to_string());

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

    assert!(
        out.contains("final int counter;"),
        "expected counter field in output:\n{out}"
    );
    assert!(
        out.find("final int counter;").unwrap() < out.find("void update").unwrap(),
        "counter field should appear before update()"
    );
}

#[test]
fn template_renders_camel_field_name_in_recipe_text() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let recipe_path = repo_root.join("test/fixtures/rust_oracle/add_counter_field.recipe.yaml");
    let recipe_yaml = std::fs::read_to_string(recipe_path).unwrap();
    let recipe = parse_recipe_yaml(&recipe_yaml).unwrap();

    let mut args = BTreeMap::new();
    args.insert("className".to_string(), "Settings".to_string());
    args.insert("field".to_string(), "MyCounter".to_string());
    args.insert("file".to_string(), "a.dart".to_string());

    let rendered = render_recipe_templates(&recipe, &args, &BTreeMap::new());
    let codemod_recipe_yaml::model::Step::Edit(edit) = &rendered.steps[0] else {
        panic!("expected edit");
    };
    let codemod_recipe_yaml::model::EditOp::Insert(insert) = &edit.ops[0] else {
        panic!("expected insert");
    };
    assert!(insert.text.contains("myCounter"));
    assert_eq!(
        render_string(&insert.query, &args).matches("Settings").count(),
        1
    );
}
