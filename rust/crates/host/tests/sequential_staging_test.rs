//! Sequential staging: create→edit, edit→edit, skip+edit, conflicts.

use codemod_recipe_host::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::registry::RecipeRegistry;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static WORKSPACE_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn temp_workspace(name: &str) -> PathBuf {
    let n = WORKSPACE_COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("seq_stage_{name}_{}_{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join(".codemod/recipes")).unwrap();
    dir
}

fn preview_inline(
    registry: &mut RecipeRegistry,
    inline: serde_json::Value,
    args: BTreeMap<String, String>,
) -> serde_json::Value {
    dispatch::handle_command(
        registry,
        HostCommand::Preview {
            recipe: None,
            inline_recipe: Some(inline),
            args,
            snippet_lines: Some(10),
        },
    )
}

#[test]
fn create_then_edit_missing_file_is_single_create() {
    let workspace = temp_workspace("create_edit_new");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!({
        "id": "ensure_barrel",
        "steps": [
            {
                "create": {
                    "path": "lib/barrel.dart",
                    "template": "// barrel\n",
                    "ifExists": "skip",
                    "format": false
                }
            },
            {
                "edit": {
                    "path": "lib/barrel.dart",
                    "ops": [{
                        "insert": {
                            "query": "(program) @root",
                            "capture": "root",
                            "anchor": "end",
                            "text": "export 'foo.dart';\n"
                        }
                    }]
                }
            }
        ]
    });

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["kind"], "create");
    assert_eq!(files[0]["path"], "lib/barrel.dart");
    assert!(files[0]["isNew"].as_bool().unwrap());
    let snippet = files[0]["snippet"].as_str().unwrap_or("");
    assert!(
        snippet.contains("export") || snippet.contains("barrel"),
        "snippet={snippet}"
    );

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn create_skip_then_edit_existing_is_patch() {
    let workspace = temp_workspace("create_skip_edit");
    std::fs::write(workspace.join("lib/barrel.dart"), "// barrel\n").unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!({
        "id": "ensure_barrel",
        "steps": [
            {
                "create": {
                    "path": "lib/barrel.dart",
                    "template": "// unused\n",
                    "ifExists": "skip",
                    "format": false
                }
            },
            {
                "edit": {
                    "path": "lib/barrel.dart",
                    "ops": [{
                        "insert": {
                            "query": "(program) @root",
                            "capture": "root",
                            "anchor": "end",
                            "text": "export 'foo.dart';\n"
                        }
                    }]
                }
            }
        ]
    });

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["kind"], "edit");
    assert_eq!(files[0]["path"], "lib/barrel.dart");
    assert!(!files[0]["isNew"].as_bool().unwrap());
    let patches = files[0]["patches"].as_array().unwrap();
    assert_eq!(patches.len(), 1);

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn edit_then_edit_dependent_class_and_constructor() {
    let workspace = temp_workspace("edit_edit");
    std::fs::write(workspace.join("lib/foo.dart"), "class Foo {\n}\n").unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!({
        "id": "class_then_ctor",
        "steps": [
            {
                "edit": {
                    "path": "lib/foo.dart",
                    "ops": [{
                        "insert": {
                            "query": "(class_definition name: (identifier) @n body: (class_body) @body (#eq? @n \"Foo\"))",
                            "capture": "body",
                            "anchor": "end",
                            "text": "  void bar() {}\n"
                        }
                    }]
                }
            },
            {
                "edit": {
                    "path": "lib/foo.dart",
                    "ops": [{
                        "insert": {
                            "query": "(class_definition name: (identifier) @className body: (class_body (method_signature (function_signature name: (identifier) @methodName)) @member) (#eq? @className \"Foo\") (#eq? @methodName \"bar\"))",
                            "capture": "member",
                            "anchor": "end",
                            "text": "\n  Foo();"
                        }
                    }]
                }
            }
        ]
    });

    let response = preview_inline(&mut registry, inline.clone(), BTreeMap::new());
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["kind"], "edit");

    // Apply and verify disk content
    let token = response["previewToken"].as_str().unwrap();
    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::Apply {
            recipe: None,
            inline_recipe: Some(inline),
            args: BTreeMap::new(),
            preview_token: token.to_string(),
            selection: serde_json::json!({}),
        },
    );
    assert_eq!(apply["ok"], true, "{}", apply["error"]);
    let src = std::fs::read_to_string(workspace.join("lib/foo.dart")).unwrap();
    assert!(src.contains("void bar()"), "{src}");
    assert!(src.contains("Foo();"), "{src}");

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn create_fail_when_exists() {
    let workspace = temp_workspace("create_fail");
    std::fs::write(workspace.join("lib/a.dart"), "x\n").unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!({
        "id": "fail_create",
        "steps": [{
            "create": {
                "path": "lib/a.dart",
                "template": "y\n",
                "ifExists": "fail",
                "format": false
            }
        }]
    });

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], false);
    assert!(
        response["error"]
            .as_str()
            .unwrap_or("")
            .contains("already exists"),
        "{}",
        response["error"]
    );

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn edit_then_create_same_path_errors() {
    let workspace = temp_workspace("edit_then_create");
    std::fs::write(workspace.join("lib/a.dart"), "class A {}\n").unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!({
        "id": "bad_order",
        "steps": [
            {
                "edit": {
                    "path": "lib/a.dart",
                    "ops": [{
                        "insert": {
                            "query": "(class_definition name: (identifier) @n body: (class_body) @body (#eq? @n \"A\"))",
                            "capture": "body",
                            "anchor": "end",
                            "text": "  // x\n"
                        }
                    }]
                }
            },
            {
                "create": {
                    "path": "lib/a.dart",
                    "template": "class A {}\n",
                    "ifExists": "fail",
                    "format": false
                }
            }
        ]
    });

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], false);
    assert!(
        response["error"]
            .as_str()
            .unwrap_or("")
            .contains("already exists"),
        "{}",
        response["error"]
    );

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn create_and_edit_different_paths() {
    let workspace = temp_workspace("multi_path");
    std::fs::write(workspace.join("lib/existing.dart"), "class E {\n}\n").unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!({
        "id": "multi",
        "steps": [
            {
                "create": {
                    "path": "lib/new.dart",
                    "template": "class N {}\n",
                    "ifExists": "fail",
                    "format": false
                }
            },
            {
                "edit": {
                    "path": "lib/existing.dart",
                    "ops": [{
                        "insert": {
                            "query": "(class_definition name: (identifier) @n body: (class_body) @body (#eq? @n \"E\"))",
                            "capture": "body",
                            "anchor": "end",
                            "text": "  // patched\n"
                        }
                    }]
                }
            }
        ]
    });

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    let kinds: Vec<_> = files.iter().map(|f| f["kind"].as_str().unwrap()).collect();
    assert!(kinds.contains(&"create"));
    assert!(kinds.contains(&"edit"));

    let _ = std::fs::remove_dir_all(workspace);
}
