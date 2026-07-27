//! Direct tests for guard / `let` query evaluation on the engine.

mod common;

use codemod_recipe_engine::engine::{EngineError, QueryContext};
use codemod_recipe_yaml::let_binding::{
    LetBinding, LetExtract, LetOnManyMatches, LetOnNoMatch,
};
use codemod_recipe_yaml::QuerySpec;
use std::path::PathBuf;

const SOURCE: &str = r#"class Demo {
  final int alpha = 1;
  final int beta = 2;
}
"#;

fn ctx() -> QueryContext<'static> {
    static ROOT: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    let root = ROOT.get_or_init(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."));
    QueryContext {
        recipe_file: None,
        codemod_root: root,
    }
}

fn class_fields_query() -> QuerySpec {
    QuerySpec::single(
        r#"(class_definition
  body: (class_body
    (declaration
      (initialized_identifier_list
        (initialized_identifier
          (identifier) @fieldName)))))
  (#eq? @fieldName "alpha")"#,
    )
}

fn two_fields_query() -> QuerySpec {
    QuerySpec::single(
        r#"(class_definition
  body: (class_body
    (declaration
      (initialized_identifier_list
        (initialized_identifier
          (identifier) @fieldName)))))"#,
    )
}

fn binding(
    extract: LetExtract,
    on_no_match: LetOnNoMatch,
    on_many: LetOnManyMatches,
    join: Option<&str>,
) -> LetBinding {
    LetBinding {
        name: "v".to_string(),
        query: Some(two_fields_query()),
        capture: Some("fieldName".to_string()),
        extract,
        on_no_match,
        on_many_matches: on_many,
        join: join.map(str::to_string),
        r#as: None,
    }
}

#[test]
fn query_has_match_true_and_false() {
    let ctx = ctx();
    common::with_engine("dart", |engine| {
        let yes = engine
            .query_has_match(&ctx, SOURCE, &class_fields_query())
            .unwrap();
        assert!(yes);
        let no = engine
            .query_has_match(
                &ctx,
                SOURCE,
                &QuerySpec::single(
                    r#"(class_definition
  name: (identifier) @n
  (#eq? @n "Missing"))"#,
                ),
            )
            .unwrap();
        assert!(!no);
    });
}

#[test]
fn let_extract_text_and_kind() {
    let ctx = ctx();
    let b_text = binding(LetExtract::Text, LetOnNoMatch::Error, LetOnManyMatches::First, None);
    let b_kind = binding(LetExtract::Kind, LetOnNoMatch::Error, LetOnManyMatches::First, None);
    common::with_engine("dart", |engine| {
        let text = engine.evaluate_let_binding(&ctx, SOURCE, &b_text).unwrap();
        assert_eq!(text, "alpha");
        let kind = engine.evaluate_let_binding(&ctx, SOURCE, &b_kind).unwrap();
        assert_eq!(kind, "identifier");
    });
}

#[test]
fn let_extract_exists_and_count() {
    let ctx = ctx();
    let exists = binding(LetExtract::Exists, LetOnNoMatch::Error, LetOnManyMatches::Error, None);
    let count = binding(LetExtract::Count, LetOnNoMatch::Error, LetOnManyMatches::Error, None);
    common::with_engine("dart", |engine| {
        assert_eq!(
            engine.evaluate_let_binding(&ctx, SOURCE, &exists).unwrap(),
            "true"
        );
        assert_eq!(
            engine.evaluate_let_binding(&ctx, SOURCE, &count).unwrap(),
            "2"
        );
    });
}

#[test]
fn let_on_no_match_use_empty() {
    let ctx = ctx();
    let mut b = binding(LetExtract::Text, LetOnNoMatch::UseEmpty, LetOnManyMatches::First, None);
    b.query = Some(QuerySpec::single(
        r#"(class_definition
  name: (identifier) @n
  (#eq? @n "Nope"))"#,
    ));
    b.capture = Some("n".to_string());
    common::with_engine("dart", |engine| {
        let v = engine.evaluate_let_binding(&ctx, SOURCE, &b).unwrap();
        assert_eq!(v, "");
    });
}

#[test]
fn let_on_no_match_errors_by_default() {
    let ctx = ctx();
    let mut b = binding(LetExtract::Text, LetOnNoMatch::Error, LetOnManyMatches::First, None);
    b.query = Some(QuerySpec::single(
        r#"(class_definition
  name: (identifier) @n
  (#eq? @n "Nope"))"#,
    ));
    b.capture = Some("n".to_string());
    common::with_engine("dart", |engine| {
        let err = engine.evaluate_let_binding(&ctx, SOURCE, &b).unwrap_err();
        assert!(matches!(err, EngineError::NoMatch { .. }));
    });
}

#[test]
fn let_on_many_matches_first_join_and_error() {
    let ctx = ctx();
    let first = binding(LetExtract::Text, LetOnNoMatch::Error, LetOnManyMatches::First, None);
    let join = binding(
        LetExtract::Text,
        LetOnNoMatch::Error,
        LetOnManyMatches::Join,
        Some("|"),
    );
    let err_binding = binding(
        LetExtract::Text,
        LetOnNoMatch::Error,
        LetOnManyMatches::Error,
        None,
    );
    common::with_engine("dart", |engine| {
        assert_eq!(
            engine.evaluate_let_binding(&ctx, SOURCE, &first).unwrap(),
            "alpha"
        );
        assert_eq!(
            engine.evaluate_let_binding(&ctx, SOURCE, &join).unwrap(),
            "alpha|beta"
        );
        let err = engine
            .evaluate_let_binding(&ctx, SOURCE, &err_binding)
            .unwrap_err();
        assert!(matches!(err, EngineError::MultipleMatches { .. }));
    });
}
