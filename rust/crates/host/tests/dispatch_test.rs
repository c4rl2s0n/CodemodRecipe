use codemod_recipe_host::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::registry::RecipeRegistry;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static WORKSPACE_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn temp_workspace(name: &str) -> PathBuf {
    let n = WORKSPACE_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("{name}_{}_{n}", std::process::id()))
}

fn setup_workspace_with_insert_recipe(name: &str) -> PathBuf {
    let workspace = temp_workspace(name);
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::copy(
        repo_root().join(".codemod/recipes/insert_log_line.yaml"),
        recipes_dir.join("insert_log_line.yaml"),
    )
    .unwrap();
    workspace
}

#[test]
fn list_reports_loaded_maps_from_repo() {
    let repo = repo_root();
    let mut registry = RecipeRegistry::new(repo.clone(), repo.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(response["ok"], true);
    assert!(response["mapsLoaded"].as_u64().unwrap_or(0) >= 1);
}

#[test]
fn list_returns_registered_recipes() {
    let workspace = setup_workspace_with_insert_recipe("list");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(response["ok"], true);
    let recipes = response["recipes"].as_array().unwrap();
    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0]["id"], "insert_log_line");

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn describe_returns_recipe_schema() {
    let workspace = setup_workspace_with_insert_recipe("describe");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::Describe {
            recipe: "insert_log_line".to_string(),
        },
    );
    assert_eq!(response["ok"], true);
    assert_eq!(response["recipe"]["id"], "insert_log_line");

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn preview_returns_preview_token_and_modified_content() {
    let workspace = setup_workspace_with_insert_recipe("preview");
    let settings = workspace.join("test/fixtures/ast_paths/settings.dart");
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/ast_paths/settings.dart"),
        &settings,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert(
        "file".to_string(),
        "test/fixtures/ast_paths/settings.dart".to_string(),
    );

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("insert_log_line".to_string()),
            inline_recipe: None,
            args,
            snippet_lines: None,
        },
    );

    assert_eq!(response["ok"], true);
    assert!(response["previewToken"]
        .as_str()
        .is_some_and(|t| !t.is_empty()));
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    let patches = files[0]["patches"].as_array().unwrap();
    assert!(!patches.is_empty());

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn preview_reports_missing_required_arguments() {
    let workspace = setup_workspace_with_insert_recipe("missing_args");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("insert_log_line".to_string()),
            inline_recipe: None,
            args: BTreeMap::new(),
            snippet_lines: None,
        },
    );

    assert_eq!(response["ok"], false);
    assert!(response["error"]
        .as_str()
        .unwrap()
        .contains("Missing required arguments"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn reload_detects_duplicate_recipe_ids() {
    let workspace = temp_workspace("codemod_host_reload");
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();

    std::fs::copy(
        repo_root().join("test/fixtures/rust_oracle/insert_log_line.recipe.yaml"),
        recipes_dir.join("insert_log_line.yaml"),
    )
    .unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/rust_oracle/duplicate_insert_log_line.recipe.yaml"),
        recipes_dir.join("duplicate_insert_log_line.yaml"),
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();
    let initial = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(initial["recipes"].as_array().unwrap().len(), 1);

    let reloaded = dispatch::handle_command(&mut registry, HostCommand::Reload);
    assert_eq!(reloaded["ok"], true);
    let diagnostics = reloaded["diagnostics"].as_array().unwrap();
    assert!(diagnostics.iter().any(|d| d["code"] == "E_DUPLICATE_ID"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn apply_writes_transformed_file() {
    let workspace = setup_workspace_with_insert_recipe("apply");
    let rel = "lib/settings.dart";
    let settings = workspace.join(rel);
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/ast_paths/settings.dart"),
        &settings,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("file".to_string(), rel.to_string());

    let preview = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("insert_log_line".to_string()),
            inline_recipe: None,
            args: args.clone(),
            snippet_lines: None,
        },
    );
    assert_eq!(preview["ok"], true);

    let token = preview["previewToken"].as_str().unwrap().to_string();
    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::Apply {
            recipe: Some("insert_log_line".to_string()),
            inline_recipe: None,
            args,
            preview_token: token,
            selection: serde_json::json!({}),
        },
    );
    assert_eq!(apply["ok"], true);

    let content = std::fs::read_to_string(&settings).unwrap();
    assert!(content.contains("print('codemod');"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn remove_is_idempotent_on_second_preview() {
    let workspace = temp_workspace("codemod_host_remove_idempotent");
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/rust_oracle/remove_count_field.recipe.yaml"),
        recipes_dir.join("remove_count_field.yaml"),
    )
    .unwrap();

    let rel = "lib/settings.dart";
    let settings = workspace.join(rel);
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(
        &settings,
        "class Settings {\n  final int count = 0;\n  final String name = 'x';\n}\n",
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("file".to_string(), rel.to_string());

    let first = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("remove_count_field".to_string()),
            inline_recipe: None,
            args: args.clone(),
            snippet_lines: None,
        },
    );
    assert_eq!(first["ok"], true);

    let token = first["previewToken"].as_str().unwrap();
    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::Apply {
            recipe: Some("remove_count_field".to_string()),
            inline_recipe: None,
            args: args.clone(),
            preview_token: token.to_string(),
            selection: serde_json::json!({}),
        },
    );
    assert_eq!(apply["ok"], true);

    let second = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("remove_count_field".to_string()),
            inline_recipe: None,
            args,
            snippet_lines: None,
        },
    );
    assert_eq!(second["ok"], true);
    let files = second["files"].as_array().unwrap();
    assert!(files.is_empty());

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn apply_rejects_missing_preview_token() {
    let workspace = setup_workspace_with_insert_recipe("apply_no_token");
    let rel = "lib/settings.dart";
    let settings = workspace.join(rel);
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/ast_paths/settings.dart"),
        &settings,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("file".to_string(), rel.to_string());

    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::Apply {
            recipe: Some("insert_log_line".to_string()),
            inline_recipe: None,
            args,
            preview_token: String::new(),
            selection: serde_json::json!({}),
        },
    );
    assert_eq!(apply["ok"], false);
    assert!(apply["error"].as_str().unwrap().contains("previewToken"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn apply_rejects_stale_preview_token_after_file_changes() {
    let workspace = setup_workspace_with_insert_recipe("apply_stale_token");
    let rel = "lib/settings.dart";
    let settings = workspace.join(rel);
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/ast_paths/settings.dart"),
        &settings,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("file".to_string(), rel.to_string());

    let preview = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("insert_log_line".to_string()),
            inline_recipe: None,
            args: args.clone(),
            snippet_lines: None,
        },
    );
    let token = preview["previewToken"].as_str().unwrap().to_string();

    std::fs::write(&settings, "// mutated\n").unwrap();

    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::Apply {
            recipe: Some("insert_log_line".to_string()),
            inline_recipe: None,
            args,
            preview_token: token,
            selection: serde_json::json!({}),
        },
    );
    assert_eq!(apply["ok"], false);
    assert!(apply["error"]
        .as_str()
        .unwrap()
        .contains("Stale previewToken"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn validate_reports_duplicate_recipe_ids() {
    let workspace = temp_workspace("codemod_host_validate");
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/rust_oracle/insert_log_line.recipe.yaml"),
        recipes_dir.join("insert_log_line.yaml"),
    )
    .unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/rust_oracle/duplicate_insert_log_line.recipe.yaml"),
        recipes_dir.join("duplicate_insert_log_line.yaml"),
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    let response = dispatch::handle_command(&mut registry, HostCommand::Validate);
    assert_eq!(response["ok"], false);
    let diagnostics = response["diagnostics"].as_array().unwrap();
    assert!(diagnostics.iter().any(|d| d["code"] == "E_DUPLICATE_ID"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn preview_includes_structured_patches() {
    let workspace = setup_workspace_with_insert_recipe("preview_patches");
    let rel = "test/fixtures/ast_paths/settings.dart";
    let settings = workspace.join(rel);
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/ast_paths/settings.dart"),
        &settings,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("file".to_string(), rel.to_string());

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("insert_log_line".to_string()),
            inline_recipe: None,
            args,
            snippet_lines: None,
        },
    );
    assert_eq!(response["ok"], true);
    let files = response["files"].as_array().unwrap();
    let patches = files[0]["patches"].as_array().unwrap();
    assert!(!patches.is_empty());
    assert!(patches[0]["offset"].is_number());

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn diff_returns_full_file_data() {
    let workspace = setup_workspace_with_insert_recipe("diff");
    let rel = "lib/settings.dart";
    let settings = workspace.join(rel);
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/ast_paths/settings.dart"),
        &settings,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("file".to_string(), rel.to_string());

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::Diff {
            recipe: Some("insert_log_line".to_string()),
            inline_recipe: None,
            args,
            path: rel.to_string(),
        },
    );
    assert_eq!(response["ok"], true);
    assert!(response["file"]["original"].as_str().is_some());
    assert!(response["file"]["modified"].as_str().is_some());

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn preview_add_log_line_with_class_and_method_args() {
    let repo = repo_root();
    let mut registry = RecipeRegistry::new(repo.clone(), repo.join(".codemod"));
    registry.reload();

    let rel = "test/fixtures/ast_paths/settings.dart";
    let mut args = BTreeMap::new();
    args.insert("file".to_string(), rel.to_string());
    args.insert("className".to_string(), "Settings".to_string());
    args.insert("methodName".to_string(), "update".to_string());

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("add_log_line".to_string()),
            inline_recipe: None,
            args,
            snippet_lines: Some(2),
        },
    );
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0]["snippet"].as_str().unwrap_or("").contains("print"));
}

#[test]
fn preview_create_file_step() {
    let workspace = temp_workspace("create_step");
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/rust_oracle/scaffold_widget.recipe.yaml"),
        recipes_dir.join("scaffold_widget.yaml"),
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("featurePath".to_string(), "lib/features".to_string());
    args.insert("name".to_string(), "Counter".to_string());

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("scaffold_widget".to_string()),
            inline_recipe: None,
            args: args.clone(),
            snippet_lines: None,
        },
    );
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0]["kind"], "create");
    assert!(files[0]["isNew"].as_bool().unwrap());

    let token = response["previewToken"].as_str().unwrap();
    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::Apply {
            recipe: Some("scaffold_widget".to_string()),
            inline_recipe: None,
            args,
            preview_token: token.to_string(),
            selection: serde_json::json!({}),
        },
    );
    assert_eq!(apply["ok"], true, "{}", apply["error"]);
    let created = workspace.join("lib/features/counter.dart");
    assert!(created.exists());
    assert!(std::fs::read_to_string(&created).unwrap().contains("class Counter"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn preview_delete_file_step() {
    let workspace = temp_workspace("delete_step");
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/rust_oracle/delete_legacy.recipe.yaml"),
        recipes_dir.join("delete_legacy.yaml"),
    )
    .unwrap();

    let legacy = workspace.join("lib/legacy.dart");
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    std::fs::write(&legacy, "class Legacy {}\n").unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("legacyPath".to_string(), "lib/legacy.dart".to_string());

    let preview = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("delete_legacy".to_string()),
            inline_recipe: None,
            args: args.clone(),
            snippet_lines: None,
        },
    );
    assert_eq!(preview["ok"], true, "{}", preview["error"]);
    assert_eq!(preview["files"][0]["kind"], "delete");

    let token = preview["previewToken"].as_str().unwrap();
    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::Apply {
            recipe: Some("delete_legacy".to_string()),
            inline_recipe: None,
            args,
            preview_token: token.to_string(),
            selection: serde_json::json!({}),
        },
    );
    assert_eq!(apply["ok"], true);
    assert!(!legacy.exists());

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn inline_recipe_preview_and_apply() {
    let workspace = temp_workspace("inline_recipe");
    let settings = workspace.join("lib/settings.dart");
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/ast_paths/settings.dart"),
        &settings,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline_recipe = serde_json::json!({
        "dslVersion": 2,
        "id": "inline_insert",
        "args": [{ "name": "file", "required": true }],
        "steps": [{
            "edit": {
                "path": "{{file}}",
                "ops": [{
                    "insert": {
                        "query": "(class_declaration name: (identifier) @className body: (class_body (class_member (method_signature (function_signature name: (identifier) @methodName)) (function_body (block) @body))) (#eq? @className \"Settings\") (#eq? @methodName \"update\"))",
                        "capture": "body",
                        "anchor": "end",
                        "text": "    print('inline');\n"
                    }
                }]
            }
        }]
    });

    let mut args = BTreeMap::new();
    args.insert("file".to_string(), "lib/settings.dart".to_string());

    let preview = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: None,
            inline_recipe: Some(inline_recipe.clone()),
            args: args.clone(),
            snippet_lines: None,
        },
    );
    assert_eq!(preview["ok"], true, "{}", preview["error"]);
    let token = preview["previewToken"].as_str().unwrap();

    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::Apply {
            recipe: None,
            inline_recipe: Some(inline_recipe),
            args,
            preview_token: token.to_string(),
            selection: serde_json::json!({}),
        },
    );
    assert_eq!(apply["ok"], true, "{}", apply["error"]);
    assert!(std::fs::read_to_string(&settings)
        .unwrap()
        .contains("print('inline')"));

    let _ = std::fs::remove_dir_all(workspace);
}

