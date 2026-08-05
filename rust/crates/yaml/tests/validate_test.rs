use codemod_recipe_yaml::let_binding::LetBinding;
use codemod_recipe_yaml::model::{CreateStep, EditOp, EditStep, Recipe, ScopedStep, Step};
use codemod_recipe_yaml::validate::{validate_recipe, validate_recipe_with, ValidationError};
use codemod_recipe_yaml::LetBindings;
use codemod_recipe_yaml::QuerySpec;
use serde_yaml::Value;
use std::collections::BTreeMap;

fn minimal_insert_edit(mut edit: EditStep) -> Recipe {
    if edit.ops.is_empty() {
        edit.ops = vec![EditOp::Insert(codemod_recipe_yaml::model::InsertOp {
            query: QuerySpec::single("(identifier) @x"),
            capture: "x".to_string(),
            anchor: codemod_recipe_yaml::model::InsertAnchor::End,
            text: "x".to_string(),
        })];
    }
    Recipe {
        id: "bad".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Edit(edit)],
        post_execution: vec![],
        explorer_menu: None,
    }
}

#[test]
fn rejects_insert_missing_capture() {
    let recipe = Recipe {
        id: "bad".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Edit(EditStep {
            path: "a.dart".to_string(),
            ops: vec![EditOp::Insert(codemod_recipe_yaml::model::InsertOp {
                query: QuerySpec::single("(identifier) @x"),
                capture: "".to_string(),
                anchor: codemod_recipe_yaml::model::InsertAnchor::End,
                text: "x".to_string(),
            })],
            ..Default::default()
        })],
        post_execution: vec![],
        explorer_menu: None,
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
fn rejects_insert_empty_text() {
    let recipe = Recipe {
        id: "bad".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Edit(EditStep {
            path: "a.dart".to_string(),
            ops: vec![EditOp::Insert(codemod_recipe_yaml::model::InsertOp {
                query: QuerySpec::single("(identifier) @x"),
                capture: "x".to_string(),
                anchor: codemod_recipe_yaml::model::InsertAnchor::End,
                text: "".to_string(),
            })],
            ..Default::default()
        })],
        post_execution: vec![],
        explorer_menu: None,
    };

    let errors = validate_recipe(&recipe).unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::MissingRequiredField {
            op: "insert",
            field: "text"
        }
    )));
}

#[test]
fn rejects_empty_edit_ops() {
    let recipe = Recipe {
        id: "bad".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Edit(EditStep {
            path: "a.dart".to_string(),
            ops: vec![],
            ..Default::default()
        })],
        post_execution: vec![],
        explorer_menu: None,
    };

    let errors = validate_recipe(&recipe).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::EmptyEditOps)));
}

#[test]
fn rejects_duplicate_arg_names() {
    let recipe = Recipe {
        id: "bad".to_string(),
        name: None,
        description: None,
        args: vec![
            codemod_recipe_yaml::model::Arg {
                name: "file".to_string(),
                required: true,
                input_kind: None,
                abbr: None,
                help: None,
                defaults_to: None,
                options: vec![],
                allow_custom_value: None,
                context_key: None,
                from: None,
                from_recipes: vec![],
            },
            codemod_recipe_yaml::model::Arg {
                name: "file".to_string(),
                required: false,
                input_kind: None,
                abbr: None,
                help: None,
                defaults_to: None,
                options: vec![],
                allow_custom_value: None,
                context_key: None,
                from: None,
                from_recipes: vec![],
            },
        ],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Edit(EditStep {
            path: "a.dart".to_string(),
            ops: vec![EditOp::Insert(codemod_recipe_yaml::model::InsertOp {
                query: QuerySpec::single("(identifier) @x"),
                capture: "x".to_string(),
                anchor: codemod_recipe_yaml::model::InsertAnchor::End,
                text: "x".to_string(),
            })],
            ..Default::default()
        })],
        post_execution: vec![],
        explorer_menu: None,
    };

    let errors = validate_recipe(&recipe).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::DuplicateArgName(name) if name == "file")));
}

#[test]
fn rejects_unknown_edit_op_kind() {
    let recipe = Recipe {
        id: "bad".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Edit(EditStep {
            path: "a.dart".to_string(),
            ops: vec![EditOp::Unknown("rename".to_string(), Value::Null)],
            ..Default::default()
        })],
        post_execution: vec![],
        explorer_menu: None,
    };

    let errors = validate_recipe(&recipe).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::UnsupportedOp(kind) if kind == "rename")));
}

#[test]
fn rejects_unknown_language() {
    let recipe = Recipe {
        id: "bad".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Edit(EditStep {
            path: "a.rs".to_string(),
            language: Some("not_a_real_language_xyz".to_string()),
            ops: vec![EditOp::Insert(codemod_recipe_yaml::model::InsertOp {
                query: QuerySpec::single("(identifier) @x"),
                capture: "x".to_string(),
                anchor: codemod_recipe_yaml::model::InsertAnchor::End,
                text: "x".to_string(),
            })],
            ..Default::default()
        })],
        post_execution: vec![],
        explorer_menu: None,
    };

    let errors = validate_recipe_with(&recipe, |_| false).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::LanguageNotSupported(id) if id == "not_a_real_language_xyz")));
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
fn rejects_let_name_colliding_with_recipe_arg() {
    let mut recipe = minimal_insert_edit(EditStep {
        path: "a.dart".to_string(),
        let_bindings: LetBindings(vec![LetBinding {
            name: "file".to_string(),
            query: Some(QuerySpec::single("(identifier) @x")),
            ..Default::default()
        }]),
        ..Default::default()
    });
    recipe.args = vec![codemod_recipe_yaml::model::Arg {
        name: "file".to_string(),
        required: true,
        input_kind: None,
        abbr: None,
        help: None,
        defaults_to: None,
        options: vec![],
        allow_custom_value: None,
        context_key: None,
        from: None,
        from_recipes: vec![],
    }];
    let errors = validate_recipe(&recipe).unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::LetNameCollidesWithArg(name) if name == "file"
    )));
}

#[test]
fn rejects_let_binding_without_query_or_as() {
    let recipe = minimal_insert_edit(EditStep {
        path: "a.dart".to_string(),
        let_bindings: LetBindings(vec![LetBinding {
            name: "n".to_string(),
            query: None,
            r#as: None,
            ..Default::default()
        }]),
        ..Default::default()
    });
    let errors = validate_recipe(&recipe).unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::LetBindingMissingQuery { name } if name == "n"
    )));
}

#[test]
fn rejects_empty_let_binding_name() {
    let recipe = minimal_insert_edit(EditStep {
        path: "a.dart".to_string(),
        let_bindings: LetBindings(vec![LetBinding {
            name: "   ".to_string(),
            query: Some(QuerySpec::single("(identifier) @x")),
            ..Default::default()
        }]),
        ..Default::default()
    });
    let errors = validate_recipe(&recipe).unwrap_err();
    assert!(errors.iter().any(|e| matches!(
        e,
        ValidationError::MissingRequiredField {
            op: "let",
            field: "name"
        }
    )));
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

#[test]
fn validation_error_needles_locate_fields() {
    assert_eq!(ValidationError::EmptyEditOps.needle(), "ops:");
    assert_eq!(
        ValidationError::MissingRequiredField {
            op: "insert",
            field: "capture",
        }
        .needle(),
        "capture:"
    );
    assert_eq!(
        ValidationError::DuplicateArgName("file".into()).needle(),
        "name: file"
    );
    assert_eq!(
        ValidationError::UnsupportedStep("patch".into()).needle(),
        "patch"
    );
    assert_eq!(ValidationError::IfStepMissingCondition.needle(), "if:");
    assert_eq!(ValidationError::EmptyIfSteps.needle(), "steps:");
}

#[test]
fn rejects_if_step_missing_condition() {
    let recipe = Recipe {
        id: "bad".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Scoped(ScopedStep {
            with: BTreeMap::new(),
            if_expr: None,
            if_not: None,
            steps: vec![Step::Create(CreateStep {
                path: "a.dart".to_string(),
                template: Some("class A {}".to_string()),
                template_file: None,
                if_exists: Default::default(),
                if_expr: None,
                if_not: None,
            })],
        })],
        post_execution: vec![],
        explorer_menu: None,
    };
    let errors = validate_recipe(&recipe).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::IfStepMissingCondition)));
}

#[test]
fn rejects_if_step_empty_steps() {
    let recipe = Recipe {
        id: "bad".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Scoped(ScopedStep {
            with: BTreeMap::new(),
            if_expr: Some("includeTests".to_string()),
            if_not: None,
            steps: vec![],
        })],
        post_execution: vec![],
        explorer_menu: None,
    };
    let errors = validate_recipe(&recipe).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::EmptyIfSteps)));
}

#[test]
fn accepts_if_step_with_condition() {
    let recipe = Recipe {
        id: "ok".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Scoped(ScopedStep {
            with: BTreeMap::new(),
            if_expr: Some("includeTests".to_string()),
            if_not: None,
            steps: vec![Step::Create(CreateStep {
                path: "a.dart".to_string(),
                template: Some("class A {}".to_string()),
                template_file: None,
                if_exists: Default::default(),
                if_expr: None,
                if_not: None,
            })],
        })],
        post_execution: vec![],
        explorer_menu: None,
    };
    validate_recipe(&recipe).unwrap();
}
