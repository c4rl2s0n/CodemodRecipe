use codemod_recipe_yaml::QuerySpec;

#[test]
fn deserializes_query_as_single_string() {
    let spec: QuerySpec = serde_yaml::from_str("\"(identifier) @x\"").unwrap();
    assert_eq!(spec, QuerySpec::single("(identifier) @x"));
    assert_eq!(spec.step_count(), 1);
}

#[test]
fn deserializes_query_chain_list() {
    let spec: QuerySpec = serde_yaml::from_str(
        r#"
- "(class_definition) @c"
- "(identifier) @x"
"#,
    )
    .unwrap();
    assert!(matches!(spec, QuerySpec::Chain(_)));
    assert_eq!(spec.step_count(), 2);
}

#[test]
fn deserializes_single_item_list_as_single() {
    let spec: QuerySpec = serde_yaml::from_str(r#"["(identifier) @x"]"#).unwrap();
    assert_eq!(spec, QuerySpec::single("(identifier) @x"));
}

#[test]
fn rejects_empty_query_list() {
    let err = serde_yaml::from_str::<QuerySpec>("[]").unwrap_err();
    assert!(err.to_string().contains("empty"));
}
