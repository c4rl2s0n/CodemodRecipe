use codemod_recipe_engine::engine::{Engine, QueryContext};
use codemod_recipe_yaml::model::{EditOp, EditStep, Recipe, RemoveOp, Step};

const SOURCE_WITH_DOC_FIELD: &str = "class Settings {\n  /// Count of items.\n  final int count = 0;\n  final int other = 1;\n}\n";

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
        steps: vec![Step::Edit(EditStep {
            path: "test.dart".to_string(),
            language: None,
            ops: vec![EditOp::Remove(RemoveOp {
                query: r#"(class_declaration
  name: (identifier) @className
  body: (class_body
    (class_member
      (declaration
        (initialized_identifier_list
          (initialized_identifier
            (identifier) @fieldName)))
    ) @member)
  (#eq? @className "Settings")
  (#eq? @fieldName "count"))"#
                    .to_string(),
                capture: "member".to_string(),
                include_leading_trivia: true,
            })],
        })],
        post_execution: vec![],
    };

    let mut engine = Engine::new_dart().unwrap();
    let out = engine
        .apply_recipe_to_source(&ctx, &recipe, "test.dart", SOURCE_WITH_DOC_FIELD)
        .unwrap()
        .modified;

    assert!(!out.contains("/// Count"));
    assert!(!out.contains("final int count"));
    assert!(out.contains("final int other"));
}
