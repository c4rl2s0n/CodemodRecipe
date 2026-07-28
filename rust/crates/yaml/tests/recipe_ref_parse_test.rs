use codemod_recipe_yaml::model::{parse_recipe_ref, Recipe, RecipeRef, Step};
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
            with: BTreeMap::new()
        }
    );
}
