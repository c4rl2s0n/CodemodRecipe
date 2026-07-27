use codemod_recipe_yaml::let_binding::{LetExtract, LetOnManyMatches, LetOnNoMatch};
use codemod_recipe_yaml::model::{EditStep, Recipe, Step};
use codemod_recipe_yaml::QuerySpec;

fn edit_from_yaml(yaml: &str) -> EditStep {
    let recipe: Recipe = serde_yaml::from_str(yaml).expect("recipe yaml");
    let Step::Edit(edit) = recipe.steps.into_iter().next().expect("one step") else {
        panic!("expected edit");
    };
    edit
}

#[test]
fn when_deserializes_single_string_guard() {
    let edit = edit_from_yaml(
        r#"
id: t
steps:
  - edit:
      path: a.dart
      when: "(class_definition) @c"
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: x
"#,
    );
    let when = edit.when.expect("when");
    assert_eq!(when.guards.len(), 1);
    assert_eq!(
        when.guards[0],
        QuerySpec::Single("(class_definition) @c".to_string())
    );
}

#[test]
fn when_not_deserializes_list_of_guards() {
    let edit = edit_from_yaml(
        r#"
id: t
steps:
  - edit:
      path: a.dart
      whenNot:
        - "(identifier) @a"
        - "(identifier) @b"
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: x
"#,
    );
    let when_not = edit.when_not.expect("whenNot");
    assert_eq!(when_not.guards.len(), 2);
}

#[test]
fn let_deserializes_single_binding() {
    let edit = edit_from_yaml(
        r#"
id: t
steps:
  - edit:
      path: a.dart
      let:
        name: n
        query: "(identifier) @n"
        capture: n
        extract: text
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: x
"#,
    );
    assert_eq!(edit.let_bindings.0.len(), 1);
    assert_eq!(edit.let_bindings.0[0].name, "n");
    assert_eq!(edit.let_bindings.0[0].extract, LetExtract::Text);
}

#[test]
fn let_deserializes_list_and_camel_case_fields() {
    let edit = edit_from_yaml(
        r#"
id: t
steps:
  - edit:
      path: a.dart
      let:
        - name: a
          query: "(identifier) @x"
          capture: x
          extract: count
          onNoMatch: use
          onManyMatches: join
          join: ","
        - name: b
          as: "{{ a }}"
      ops:
        - insert:
            query: "(class_definition) @c"
            capture: c
            anchor: end
            text: x
"#,
    );
    assert_eq!(edit.let_bindings.0.len(), 2);
    let first = &edit.let_bindings.0[0];
    assert_eq!(first.on_no_match, LetOnNoMatch::UseEmpty);
    assert_eq!(first.on_many_matches, LetOnManyMatches::Join);
    assert_eq!(first.join.as_deref(), Some(","));
    assert_eq!(edit.let_bindings.0[1].r#as.as_deref(), Some("{{ a }}"));
}
