use codemod_recipe_engine::ops::{EditOp, InsertAnchor, InsertOp, QuerySpec};
use codemod_recipe_engine::{
    ensure_language_downloaded, is_known_language, language_from_extension, LanguageRegistry,
    RegistryConfig,
};

mod common;

#[test]
fn lazy_loads_dart_from_language_pack() {
    ensure_language_downloaded("dart");
    let mut registry = LanguageRegistry::new();
    assert!(registry.get("dart").is_ok());
    assert!(registry.get("dart").is_ok(), "second call uses cache");
}

#[test]
fn unknown_language_returns_error() {
    let mut registry = LanguageRegistry::new();
    match registry.get("not_a_real_language_xyz") {
        Err(err) => assert!(err.to_string().contains("not_a_real_language_xyz")),
        Ok(_) => panic!("expected unknown language to fail"),
    }
}

#[test]
fn resolves_language_from_extension() {
    let config = RegistryConfig::default();
    assert_eq!(
        language_from_extension("lib/main.dart", &config).as_deref(),
        Some("dart")
    );
    assert_eq!(
        language_from_extension("src/lib.rs", &config).as_deref(),
        Some("rust")
    );
    assert_eq!(
        language_from_extension("App.java", &config).as_deref(),
        Some("java")
    );
    assert_eq!(
        language_from_extension("Main.kt", &config).as_deref(),
        Some("kotlin")
    );
    assert_eq!(
        language_from_extension("schema.sql", &config).as_deref(),
        Some("sqlite")
    );
}

#[test]
fn registry_resolves_explicit_language_over_extension() {
    ensure_language_downloaded("rust");
    let registry = LanguageRegistry::new();
    let id = registry
        .resolve_language_id(Some("rust"), "main.dart")
        .unwrap();
    assert_eq!(id, "rust");
}

#[test]
fn resolves_dart_from_extension_without_explicit_language() {
    let registry = LanguageRegistry::new();
    let id = registry.resolve_language_id(None, "lib/main.dart").unwrap();
    assert_eq!(id, "dart");
}

#[test]
fn rejects_unsupported_file_type_when_extension_unknown() {
    let registry = LanguageRegistry::new();
    match registry.resolve_language_id(None, "weird.xyz") {
        Err(err) => {
            let msg = err.to_string();
            assert!(msg.contains("file type not supported"), "{msg}");
            assert!(msg.contains("weird.xyz"), "{msg}");
        }
        Ok(id) => panic!("expected unsupported file type, got {id}"),
    }
}

#[test]
fn rejects_unsupported_file_type_when_no_extension() {
    let registry = LanguageRegistry::new();
    match registry.resolve_language_id(None, "noext") {
        Err(err) => {
            let msg = err.to_string();
            assert!(msg.contains("file type not supported"), "{msg}");
            assert!(msg.contains("noext"), "{msg}");
        }
        Ok(id) => panic!("expected unsupported file type, got {id}"),
    }
}

#[test]
fn rejects_unsupported_explicit_language() {
    let registry = LanguageRegistry::new();
    match registry.resolve_language_id(Some("not_a_real_language_xyz"), "main.rs") {
        Err(err) => {
            let msg = err.to_string();
            assert!(msg.contains("language not supported"), "{msg}");
            assert!(msg.contains("not_a_real_language_xyz"), "{msg}");
        }
        Ok(id) => panic!("expected unsupported language, got {id}"),
    }
}

#[test]
fn is_known_language_false_for_unknown() {
    assert!(!is_known_language("not_a_real_language_xyz"));
}

#[test]
fn smoke_parse_rust_java_kotlin_sql() {
    for lang in ["rust", "java", "kotlin", "sql"] {
        ensure_language_downloaded(lang);
        let mut registry = LanguageRegistry::new();
        assert!(
            registry.get(lang).is_ok(),
            "failed to load language: {lang}"
        );
    }
}

#[test]
fn smoke_sqlite_native_override() {
    let mut registry = LanguageRegistry::new();
    assert!(registry.get("sqlite").is_ok());
    assert!(is_known_language("sqlite"));
}

fn insert_ops(query: &str, capture: &str, text: &str) -> Vec<EditOp> {
    vec![EditOp::Insert(InsertOp {
        query: QuerySpec::single(query),
        capture: capture.to_string(),
        anchor: InsertAnchor::End,
        text: text.to_string(),
    })]
}

#[test]
fn rust_insert_at_function_item_end() {
    ensure_language_downloaded("rust");
    let source = "fn hello() {\n}\n";
    let ops = insert_ops("(function_item) @fn", "fn", "\n// inserted");
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let ctx = codemod_recipe_engine::engine::QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };
    let patches = common::with_engine("rust", |engine| {
        engine.collect_patches_for_ops(&ctx, &ops, source).unwrap()
    });
    assert!(!patches.is_empty());
}

#[test]
fn smoke_sqlite_parses_create_table() {
    let mut registry = LanguageRegistry::new();
    let source = "CREATE TABLE users (id INTEGER PRIMARY KEY);";
    let ops = insert_ops("(create_table_statement) @stmt", "stmt", "\n-- codemod");
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let ctx = codemod_recipe_engine::engine::QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };
    let engine = registry.get("sqlite").expect("sqlite");
    let patches = engine
        .collect_patches_for_ops(&ctx, &ops, source)
        .expect("sqlite patches");
    assert!(!patches.is_empty());
}
