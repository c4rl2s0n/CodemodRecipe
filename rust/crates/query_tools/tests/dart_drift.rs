#[cfg(test)]
mod integration {
    use codemod_recipe_query_tools::{
        debug_query, dump_ast, generate_query, DebugOptions, DumpOptions, GenerateOptions,
    };
    use pretty_assertions::assert_eq;
    use tree_sitter::Parser;

    fn dart_language() -> tree_sitter::Language {
        tree_sitter_language_pack::get_language("dart").expect("dart grammar")
    }

    fn parse(source: &str) -> tree_sitter::Tree {
        let lang = dart_language();
        let mut parser = Parser::new();
        parser.set_language(&lang).unwrap();
        parser.parse(source, None).unwrap()
    }

    const DRIFT: &str = r#"
import 'package:drift/drift.dart';

@DriftDatabase(
  tables: [
    HostsTable,
    RulesTable,
  ],
)
class AppDatabase extends _$AppDatabase {}
"#;

    #[test]
    fn dump_ast_finds_annotation() {
        let tree = parse(DRIFT);
        let dump = dump_ast(tree.root_node(), DRIFT, &DumpOptions::default());
        assert_eq!(dump.kind, "program");
        let has_annotation = dump.children.iter().any(|c| c.kind == "class_definition");
        assert!(has_annotation || dump.children.iter().any(|c| c.kind.contains("class")));
    }

    #[test]
    fn debug_query_tables_list_last_identifier() {
        let lang = dart_language();
        let query = r#"
(annotation
  name: (identifier) @annotationName
  (arguments
    (named_argument
      (label (identifier) @argName)
      (list_literal
        (identifier) @lastTable .)))
  (#eq? @annotationName "DriftDatabase")
  (#eq? @argName "tables"))
"#;
        let result = debug_query(&lang, DRIFT, query, &DebugOptions {
            instrument: false,
            ..DebugOptions::default()
        })
        .expect("query");
        assert!(!result.has_error, "parse errors in fixture");
        assert_eq!(result.match_count, 1);
        let last = result.matches[0]
            .captures
            .iter()
            .find(|c| c.name == "lastTable")
            .expect("lastTable");
        assert_eq!(last.kind, "identifier");
        assert_eq!(last.text.as_deref(), Some("RulesTable"));
    }

    #[test]
    fn generate_query_from_table_identifier() {
        let tree = parse(DRIFT);
        // Find byte offset of RulesTable
        let offset = DRIFT.find("RulesTable").expect("RulesTable");
        let gen = generate_query(
            &tree,
            DRIFT,
            offset,
            offset + "RulesTable".len(),
            &GenerateOptions {
                include_text_predicates: false,
                capture_leaf: "lastTable".into(),
                max_depth: Some(6),
            },
        )
        .expect("generate");
        assert!(gen.query.contains("list_literal") || gen.query.contains("identifier"));
        assert!(gen.query.contains("@lastTable"));
        assert!(
            gen.query.contains("@lastTable .") || gen.query.contains("@lastTable  ."),
            "expected last-child anchor . after capture: {}",
            gen.query
        );
        assert!(!gen.query.contains("#eq?"), "{}", gen.query);
        assert_eq!(gen.capture_suggestion, "lastTable");
    }

    #[test]
    fn generate_query_pins_text_when_requested() {
        let tree = parse(DRIFT);
        let offset = DRIFT.find("RulesTable").expect("RulesTable");
        let gen = generate_query(
            &tree,
            DRIFT,
            offset,
            offset + "RulesTable".len(),
            &GenerateOptions {
                include_text_predicates: true,
                capture_leaf: "lastTable".into(),
                max_depth: Some(6),
            },
        )
        .expect("generate");
        assert!(gen.query.contains("#eq? @lastTable \"RulesTable\""));
        assert!(
            !gen.query.contains("@lastTable ."),
            "pin text should omit last-child .: {}",
            gen.query
        );
    }
}
