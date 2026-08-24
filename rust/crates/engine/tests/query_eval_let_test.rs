//! Engine query evaluation helpers.

mod common;

use codemod_recipe_engine::engine::QueryContext;
use codemod_recipe_engine::ops::QuerySpec;
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
fn query_capture_spans_and_count() {
    let ctx = ctx();
    common::with_engine("dart", |engine| {
        let spans = engine
            .query_capture_spans(&ctx, SOURCE, &two_fields_query(), "fieldName")
            .unwrap();
        assert_eq!(spans.len(), 2);
        let count = engine
            .query_match_count(&ctx, SOURCE, &two_fields_query(), Some("fieldName"))
            .unwrap();
        assert_eq!(count, 2);
        let text0 = &SOURCE[spans[0].start..spans[0].end];
        assert_eq!(text0, "alpha");
    });
}
