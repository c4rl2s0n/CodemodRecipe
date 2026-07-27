//! Query chain + query library resolution.

use codemod_recipe_host::registry::RecipeRegistry;
use codemod_recipe_host::runner::collect_recipe_changes;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn chained_query_inserts_into_method_body() {
    let workspace = repo_root();
    let codemod = workspace.join(".codemod");
    let mut registry = RecipeRegistry::new(workspace.clone(), codemod);
    registry.reload();

    assert!(registry.queries_by_id().contains_key("dart_queries"));

    let inline = serde_json::json!({
        "dslVersion": 2,
        "id": "chain_test",
        "args": [
            { "name": "className", "required": true },
            { "name": "methodName", "required": true }
        ],
        "steps": [{
            "edit": {
                "path": "lib/foo.dart",
                "ops": [{
                    "insert": {
                        "query": [
                            "dart_queries.class_named",
                            "dart_queries.method_body"
                        ],
                        "capture": "body",
                        "anchor": "end",
                        "text": "    print('codemod');\n"
                    }
                }]
            }
        }]
    });

    let recipe = codemod_recipe_host::runner::parse_inline_recipe(&inline).unwrap();
    let args = BTreeMap::from([
        ("className".to_string(), "Settings".to_string()),
        ("methodName".to_string(), "update".to_string()),
    ]);

    let source = "class Settings {\n  void update() {}\n}\n";
    std::fs::create_dir_all(workspace.join("lib")).ok();
    std::fs::write(workspace.join("lib/foo.dart"), source).unwrap();

    let collected = collect_recipe_changes(&registry, &recipe, None, &args).unwrap();
    let modified = collected
        .changes
        .iter()
        .find_map(|c| c.modified_content().ok().flatten())
        .expect("patch");

    assert!(modified.contains("print('codemod')"));

    let _ = std::fs::remove_file(workspace.join("lib/foo.dart"));
}

#[test]
fn chained_query_mixed_library_ref_and_inline_step() {
    let workspace = repo_root();
    let codemod = workspace.join(".codemod");
    let mut registry = RecipeRegistry::new(workspace.clone(), codemod);
    registry.reload();

    let inline = serde_json::json!({
        "dslVersion": 2,
        "id": "chain_mixed",
        "args": [
            { "name": "className", "required": true },
            { "name": "methodName", "required": true }
        ],
        "steps": [{
            "edit": {
                "path": "lib/foo.dart",
                "ops": [{
                    "insert": {
                        "query": [
                            "dart_queries.class_named",
                            "(function_body (block) @body)"
                        ],
                        "capture": "body",
                        "anchor": "end",
                        "text": "    print('mixed');\n"
                    }
                }]
            }
        }]
    });

    let recipe = codemod_recipe_host::runner::parse_inline_recipe(&inline).unwrap();
    let args = BTreeMap::from([
        ("className".to_string(), "Settings".to_string()),
        ("methodName".to_string(), "update".to_string()),
    ]);

    let source = "class Settings {\n  void update() {}\n}\n";
    std::fs::create_dir_all(workspace.join("lib")).ok();
    std::fs::write(workspace.join("lib/foo.dart"), source).unwrap();

    let collected = collect_recipe_changes(&registry, &recipe, None, &args).unwrap();
    let modified = collected
        .changes
        .iter()
        .find_map(|c| c.modified_content().ok().flatten())
        .expect("patch");

    assert!(modified.contains("print('mixed')"));

    let _ = std::fs::remove_file(workspace.join("lib/foo.dart"));
}
