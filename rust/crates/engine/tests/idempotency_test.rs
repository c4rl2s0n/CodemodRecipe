use codemod_recipe_engine::engine::{parse_recipe_yaml, Engine, QueryContext};
use codemod_recipe_yaml::model::{EditOp, EditStep, Recipe, RemoveOp, ReplaceOp, Step};

const SETTINGS: &str = "class Settings {\n  final int count = 0;\n}\n";

const SETTINGS_WITHOUT_COUNT: &str = "class Settings {\n  \n}\n";

const FIELD_QUERY: &str = r#"(class_declaration
  name: (identifier) @className
  body: (class_body
    (class_member
      (declaration
        (initialized_identifier_list
          (initialized_identifier
            (identifier) @fieldName)))
    ) @member)
  (#eq? @className "Settings")
  (#eq? @fieldName "count"))"#;

fn inline_ctx<'a>(codemod_root: &'a std::path::Path) -> QueryContext<'a> {
    QueryContext {
        recipe_file: None,
        codemod_root,
    }
}

fn remove_count_recipe() -> Recipe {
    Recipe {
        id: "remove_count".to_string(),
        name: None,
        description: None,
        args: vec![],
        steps: vec![Step::Edit(EditStep {
            path: "test.dart".to_string(),
            language: None,
            ops: vec![EditOp::Remove(RemoveOp {
                query: FIELD_QUERY.to_string(),
                capture: "member".to_string(),
                include_leading_trivia: false,
            })],
        })],
        post_execution: vec![],
    }
}

fn replace_count_recipe(text: &str) -> Recipe {
    Recipe {
        id: "replace_count".to_string(),
        name: None,
        description: None,
        args: vec![],
        steps: vec![Step::Edit(EditStep {
            path: "test.dart".to_string(),
            language: None,
            ops: vec![EditOp::Replace(ReplaceOp {
                query: FIELD_QUERY.to_string(),
                capture: "member".to_string(),
                text: text.to_string(),
                include_leading_trivia: false,
            })],
        })],
        post_execution: vec![],
    }
}

#[test]
fn remove_deletes_field_declaration() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let mut engine = Engine::new_dart().unwrap();
    let recipe = remove_count_recipe();
    let ctx = inline_ctx(&codemod);
    let out = engine
        .apply_recipe_to_source(&ctx, &recipe, "test.dart", SETTINGS)
        .unwrap()
        .modified;
    assert_eq!(out, SETTINGS_WITHOUT_COUNT);
}

#[test]
fn remove_no_ops_when_field_absent() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let mut engine = Engine::new_dart().unwrap();
    let recipe = remove_count_recipe();
    let ctx = inline_ctx(&codemod);
    let out = engine
        .apply_recipe_to_source(&ctx, &recipe, "test.dart", SETTINGS_WITHOUT_COUNT)
        .unwrap()
        .modified;
    assert_eq!(out, SETTINGS_WITHOUT_COUNT);
}

#[test]
fn replace_no_ops_when_whitespace_normalized_text_matches() {
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let mut engine = Engine::new_dart().unwrap();
    let recipe = replace_count_recipe("final int count = 0;");
    let ctx = inline_ctx(&codemod);
    let out = engine
        .apply_recipe_to_source(&ctx, &recipe, "test.dart", SETTINGS)
        .unwrap()
        .modified;
    assert_eq!(out, SETTINGS);
}

#[test]
fn golden_remove_count_field_fixture_parses() {
    let repo_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let recipe_path = repo_root.join("test/fixtures/rust_oracle/remove_count_field.recipe.yaml");
    let text = std::fs::read_to_string(recipe_path).unwrap();
    let recipe = parse_recipe_yaml(&text).unwrap();
    assert_eq!(recipe.id, "remove_count_field");
}
