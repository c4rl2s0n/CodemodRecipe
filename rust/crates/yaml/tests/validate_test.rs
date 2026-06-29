use codemod_recipe_yaml::model::{EditOp, EditStep, Recipe, Step};
use codemod_recipe_yaml::validate::{validate_recipe, ValidationError};
use serde_yaml::Value;

#[test]
fn rejects_insert_missing_capture() {
    let recipe = Recipe {
        id: "bad".to_string(),
        name: None,
        description: None,
        args: vec![],
        steps: vec![Step::Edit(EditStep {
            path: "a.dart".to_string(),
            language: None,
            ops: vec![EditOp::Insert(codemod_recipe_yaml::model::InsertOp {
                query: "(identifier) @x".to_string(),
                capture: "".to_string(),
                anchor: codemod_recipe_yaml::model::InsertAnchor::End,
                text: "x".to_string(),
            })],
        })],
        post_execution: vec![],
    };

    let errors = validate_recipe(&recipe).unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::MissingRequiredField {
            op: "insert",
            field: "capture"
        }
    )));
}

#[test]
fn rejects_unknown_edit_op_kind() {
    let recipe = Recipe {
        id: "bad".to_string(),
        name: None,
        description: None,
        args: vec![],
        steps: vec![Step::Edit(EditStep {
            path: "a.dart".to_string(),
            language: None,
            ops: vec![EditOp::Unknown("rename".to_string(), Value::Null)],
        })],
        post_execution: vec![],
    };

    let errors = validate_recipe(&recipe).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::UnsupportedOp(kind) if kind == "rename")));
}

#[test]
fn accepts_insert_replace_remove_ops() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../test/fixtures/rust_oracle/insert_log_line.recipe.yaml");
    let text = std::fs::read_to_string(fixture_path).unwrap();
    let recipe: Recipe = serde_yaml::from_str(&text).unwrap();
    validate_recipe(&recipe).unwrap();
}

#[test]
fn parses_codemod_insert_log_line_recipe() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../.codemod/recipes/insert_log_line.yaml");
    let text = std::fs::read_to_string(fixture_path).unwrap();
    let recipe: Recipe = serde_yaml::from_str(&text).unwrap();
    assert_eq!(recipe.id, "insert_log_line");
    assert_eq!(recipe.args.len(), 1);
    assert_eq!(recipe.args[0].name, "file");
}

#[test]
fn parses_remove_count_oracle_recipe() {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../test/fixtures/rust_oracle/remove_count_field.recipe.yaml");
    let text = std::fs::read_to_string(fixture_path).unwrap();
    let recipe: Recipe = serde_yaml::from_str(&text).unwrap();
    assert_eq!(recipe.id, "remove_count_field");
    let Step::Edit(edit) = &recipe.steps[0] else {
        panic!("expected edit step");
    };
    assert!(matches!(edit.ops[0], EditOp::Remove(_)));
}
