use codemod_recipe_yaml::model::{parse_recipe_ref, CreateStep, EditStep, Recipe, RecipeRef, Step};
use std::collections::BTreeMap;

#[test]
fn parses_recipe_ref_string() {
    let recipe: Recipe = serde_yaml::from_str(
        r#"
id: parent
steps:
  - recipe: child_id
"#,
    )
    .unwrap();
    let Step::RecipeRef(r) = &recipe.steps[0] else {
        panic!("expected RecipeRef");
    };
    assert_eq!(r.id, "child_id");
    assert!(r.with.is_empty());
    assert!(!r.has_condition());
}

#[test]
fn parses_recipe_ref_with_bindings() {
    let recipe: Recipe = serde_yaml::from_str(
        r#"
id: parent
steps:
  - recipe:
      id: child_id
      with:
        className: "{{ featureName }}"
        suffix: "Widget"
"#,
    )
    .unwrap();
    let Step::RecipeRef(r) = &recipe.steps[0] else {
        panic!("expected RecipeRef");
    };
    assert_eq!(r.id, "child_id");
    assert_eq!(r.with.get("className").unwrap(), "{{ featureName }}");
    assert_eq!(r.with.get("suffix").unwrap(), "Widget");
}

#[test]
fn parses_recipe_ref_with_if_and_if_not() {
    let recipe: Recipe = serde_yaml::from_str(
        r#"
id: parent
steps:
  - recipe:
      id: child_id
      if: includeTests
      ifNot: file | file_exists
"#,
    )
    .unwrap();
    let Step::RecipeRef(r) = &recipe.steps[0] else {
        panic!("expected RecipeRef");
    };
    assert_eq!(r.id, "child_id");
    assert_eq!(r.if_expr.as_deref(), Some("includeTests"));
    assert_eq!(r.if_not.as_deref(), Some("file | file_exists"));
    assert!(r.has_condition());
}

#[test]
fn parses_edit_and_create_if_fields() {
    let recipe: Recipe = serde_yaml::from_str(
        r#"
id: gated
steps:
  - create:
      path: "lib/a.dart"
      template: "class A {}"
      if: kind == "widget"
  - edit:
      path: "lib/a.dart"
      ifNot: skipEdit
      ops:
        - insert:
            query: "(program) @root"
            capture: root
            anchor: end
            text: "// x\n"
"#,
    )
    .unwrap();
    let Step::Create(CreateStep { if_expr, .. }) = &recipe.steps[0] else {
        panic!("expected Create");
    };
    assert_eq!(if_expr.as_deref(), Some("kind == \"widget\""));
    let Step::Edit(EditStep { if_not, .. }) = &recipe.steps[1] else {
        panic!("expected Edit");
    };
    assert_eq!(if_not.as_deref(), Some("skipEdit"));
}

#[test]
fn parse_recipe_ref_rejects_unknown_field() {
    let err = parse_recipe_ref(
        serde_yaml::from_str(
            r#"
id: child
extra: true
"#,
        )
        .unwrap(),
    )
    .unwrap_err();
    assert!(err.contains("unknown field"));
}

#[test]
fn parse_recipe_ref_string_helper() {
    let r = parse_recipe_ref(serde_yaml::Value::String("x".into())).unwrap();
    assert_eq!(
        r,
        RecipeRef {
            id: "x".into(),
            with: BTreeMap::new(),
            if_expr: None,
            if_not: None,
        }
    );
}
