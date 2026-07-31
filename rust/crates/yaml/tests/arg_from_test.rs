#[cfg(test)]
mod tests {
    use codemod_recipe_yaml::arg_from::{ArgFrom, ArgFromScope, ArgFromSpec};
    use codemod_recipe_yaml::model::Arg;

    #[test]
    fn deserializes_string_from() {
        let arg: Arg = serde_yaml::from_str(
            r#"
name: file
from: file
"#,
        )
        .unwrap();
        assert_eq!(arg.from, Some(ArgFrom::Builtin("file".into())));
    }

    #[test]
    fn deserializes_query_from() {
        let arg: Arg = serde_yaml::from_str(
            r#"
name: className
from:
  query: "(class_definition name: (identifier) @name)"
  capture: name
  scope: enclosing
"#,
        )
        .unwrap();
        match arg.from {
            Some(ArgFrom::Spec(ArgFromSpec {
                query: Some(_),
                capture: Some(cap),
                scope: Some(ArgFromScope::Enclosing),
                ..
            })) => assert_eq!(cap, "name"),
            other => panic!("unexpected from: {other:?}"),
        }
    }

    #[test]
    fn context_key_still_parses() {
        let arg: Arg = serde_yaml::from_str(
            r#"
name: word
contextKey: word
"#,
        )
        .unwrap();
        assert_eq!(arg.context_key.as_deref(), Some("word"));
        assert!(arg.from.is_none());
    }
}
