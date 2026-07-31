use codemod_recipe_engine::{
    ensure_language_downloaded, is_known_language, language_from_extension, LanguageRegistry,
    RegistryConfig,
};
use codemod_recipe_yaml::model::{EditOp, EditStep, InsertAnchor, InsertOp, Recipe, Step};
use codemod_recipe_yaml::validate::{validate_recipe_with, ValidationError};
use codemod_recipe_yaml::QuerySpec;
use std::collections::BTreeMap;

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
fn validates_unknown_explicit_language() {
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
            ops: vec![EditOp::Insert(InsertOp {
                query: QuerySpec::single("(identifier) @x"),
                capture: "x".to_string(),
                anchor: InsertAnchor::End,
                text: "x".to_string(),
            })],
            ..Default::default()
        })],
        post_execution: vec![],
        explorer_menu: None,
    };

    let errors = validate_recipe_with(&recipe, is_known_language).unwrap_err();
    assert!(errors
        .iter()
        .any(|e| matches!(e, ValidationError::LanguageNotSupported(id) if id == "not_a_real_language_xyz")));
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

#[test]
fn rust_insert_at_function_item_end() {
    ensure_language_downloaded("rust");
    let source = "fn hello() {\n}\n";
    let recipe = Recipe {
        id: "rust_insert".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Edit(EditStep {
            path: "main.rs".to_string(),
            language: Some("rust".to_string()),
            ops: vec![EditOp::Insert(InsertOp {
                query: QuerySpec::single("(function_item) @fn"),
                capture: "fn".to_string(),
                anchor: InsertAnchor::End,
                text: "\n// inserted".to_string(),
            })],
            ..Default::default()
        })],
        post_execution: vec![],
        explorer_menu: None,
    };

    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let ctx = codemod_recipe_engine::engine::QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };

    let out = common::with_engine("rust", |engine| {
        let edit = match &recipe.steps[0] {
            Step::Edit(edit) => edit,
            _ => panic!("edit step"),
        };
        engine.collect_patches_for_edit(&ctx, edit, source).unwrap()
    });

    assert!(!out.is_empty());
}

#[test]
fn smoke_sqlite_parses_create_table() {
    let mut registry = LanguageRegistry::new();
    let source = "CREATE TABLE users (id INTEGER PRIMARY KEY);";
    let recipe = Recipe {
        id: "sqlite_create".to_string(),
        name: None,
        description: None,
        args: vec![],
        maps: BTreeMap::new(),
        queries: BTreeMap::new(),
        steps: vec![Step::Edit(EditStep {
            path: "schema.sql".to_string(),
            language: Some("sqlite".to_string()),
            ops: vec![EditOp::Insert(InsertOp {
                query: QuerySpec::single("(create_table_statement) @stmt"),
                capture: "stmt".to_string(),
                anchor: InsertAnchor::End,
                text: "\n-- codemod".to_string(),
            })],
            ..Default::default()
        })],
        post_execution: vec![],
        explorer_menu: None,
    };
    let repo = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let codemod = repo.join(".codemod");
    let ctx = codemod_recipe_engine::engine::QueryContext {
        recipe_file: None,
        codemod_root: &codemod,
    };
    let engine = registry.get("sqlite").expect("sqlite");
    let edit = match &recipe.steps[0] {
        Step::Edit(edit) => edit,
        _ => panic!("edit"),
    };
    let patches = engine
        .collect_patches_for_edit(&ctx, edit, source)
        .expect("sqlite patches");
    assert!(!patches.is_empty());
}
