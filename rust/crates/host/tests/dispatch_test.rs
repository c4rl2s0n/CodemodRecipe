mod common;

use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::keys as protocol_keys;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use common::{repo_root, temp_workspace};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn setup_workspace_with_insert_recipe(name: &str) -> PathBuf {
    let workspace = temp_workspace(name);
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/dart_demo/.codemod/recipes/insert_log_line.luau"),
        recipes_dir.join("insert_log_line.luau"),
    )
    .unwrap();
    workspace
}

#[test]
fn list_reports_loaded_maps_from_fixture() {
    let fixture = repo_root().join("test/fixtures/dart_demo");
    let mut registry = RecipeRegistry::new(fixture.clone(), fixture.join(".codemod"));
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
fn list_includes_query_ids_when_query_libs_present() {
    let workspace = temp_workspace("list_query_ids");
    let queries_dir = workspace.join(".codemod/queries");
    std::fs::create_dir_all(&queries_dir).unwrap();
    std::fs::write(
        queries_dir.join("demo.luau"),
        r#"return {
  id = "demo_queries",
  queries = {
    body = [[(block) @body]],
  },
}
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();
    let response = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(response["ok"], true);
    let ids = response["queryIds"].as_array().expect("queryIds");
    assert!(
        ids.iter().any(|v| v.as_str() == Some("demo_queries.body")),
        "expected demo_queries.body in {ids:?}"
    );
    assert!(response["queries"]["demo_queries.body"]
        .as_str()
        .unwrap()
        .contains("@body"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn query_captures_from_inline_query() {
    let workspace = temp_workspace("query_captures_inline");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::QueryCaptures {
            query: Some("(class_definition name: (identifier) @className) @cls".into()),
            query_id: None,
        },
    );
    assert_eq!(response["ok"], true);
    let captures = response["captures"].as_array().unwrap();
    let names: Vec<&str> = captures.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(names, vec!["className", "cls"]);

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn query_captures_from_registered_query_id() {
    let workspace = temp_workspace("query_captures_id");
    let queries_dir = workspace.join(".codemod/queries");
    std::fs::create_dir_all(&queries_dir).unwrap();
    std::fs::write(
        queries_dir.join("demo.luau"),
        r#"return {
  id = "demo_queries",
  queries = {
    method = [[
(function_body (block) @body)
(identifier) @name
]],
  },
}
"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::QueryCaptures {
            query: None,
            query_id: Some("demo_queries.method".into()),
        },
    );
    assert_eq!(response["ok"], true);
    let captures = response["captures"].as_array().unwrap();
    let names: Vec<&str> = captures.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(names, vec!["body", "name"]);

    let missing = dispatch::handle_command(
        &mut registry,
        HostCommand::QueryCaptures {
            query: None,
            query_id: Some("nope.missing".into()),
        },
    );
    assert_eq!(missing["ok"], false);

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
        serde_json::Value::String("test/fixtures/ast_paths/settings.dart".to_string()),
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
    let files = response[protocol_keys::FILES].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0]["original"].as_str().is_some());
    assert!(files[0]["modified"].as_str().is_some());
    assert!(files[0]["patches"].is_null());

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
        repo_root().join("test/fixtures/rust_oracle/insert_log_line.recipe.luau"),
        recipes_dir.join("insert_log_line.luau"),
    )
    .unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/rust_oracle/duplicate_insert_log_line.recipe.luau"),
        recipes_dir.join("duplicate_insert_log_line.luau"),
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();
    let initial = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(initial["recipes"].as_array().unwrap().len(), 1);

    let reloaded = dispatch::handle_command(&mut registry, HostCommand::Reload);
    assert_eq!(reloaded["ok"], true);
    let diagnostics = reloaded["diagnostics"].as_array().unwrap();
    assert!(diagnostics.iter().any(|d| d["code"] == "E_DUP_ID"));

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
    args.insert(
        "file".to_string(),
        serde_json::Value::String(rel.to_string()),
    );

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
    assert_eq!(apply["ok"], true, "{}", apply["error"]);

    let content = std::fs::read_to_string(&settings).unwrap();
    assert!(content.contains("print('codemod');"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn apply_runs_post_execution_rustfmt() {
    let workspace = temp_workspace("post_exec_rustfmt");
    std::fs::create_dir_all(workspace.join(".codemod/recipes")).unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let rel = "src/main.rs";
    let inline_recipe = serde_json::json!(
        r#"
return {
  id = "post_exec_rustfmt",
  args = { { name = "file", required = true } },
  postExecution = { "rustfmt {{file}}" },
  apply = function(c, args)
    c:create({
      path = args.file,
      content = c:template({ inline = "fn main(){println!(\"hi\");}\n" }),
    })
  end,
}
"#
    );

    let mut args = BTreeMap::new();
    args.insert(
        "file".to_string(),
        serde_json::Value::String(rel.to_string()),
    );

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

    let content = std::fs::read_to_string(workspace.join(rel)).unwrap();
    assert_eq!(
        content, "fn main() {\n    println!(\"hi\");\n}\n",
        "postExecution rustfmt should reformat the created file"
    );

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn remove_is_idempotent_on_second_preview() {
    let workspace = temp_workspace("codemod_host_remove_idempotent");
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/rust_oracle/remove_count_field.recipe.luau"),
        recipes_dir.join("remove_count_field.luau"),
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
    args.insert(
        "file".to_string(),
        serde_json::Value::String(rel.to_string()),
    );

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
    let files = second[protocol_keys::FILES].as_array().unwrap();
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
    args.insert(
        "file".to_string(),
        serde_json::Value::String(rel.to_string()),
    );

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
    args.insert(
        "file".to_string(),
        serde_json::Value::String(rel.to_string()),
    );

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

    // Bump mtime without changing semantic content so preview can re-run but token is stale.
    let original = std::fs::read_to_string(&settings).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(10));
    std::fs::write(&settings, original).unwrap();

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
        repo_root().join("test/fixtures/rust_oracle/insert_log_line.recipe.luau"),
        recipes_dir.join("insert_log_line.luau"),
    )
    .unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/rust_oracle/duplicate_insert_log_line.recipe.luau"),
        recipes_dir.join("duplicate_insert_log_line.luau"),
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    let response = dispatch::handle_command(&mut registry, HostCommand::Validate { recipe: None });
    assert_eq!(response["ok"], false);
    let diagnostics = response["diagnostics"].as_array().unwrap();
    assert!(diagnostics.iter().any(|d| d["code"] == "E_DUP_ID"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn preview_includes_original_and_modified() {
    let workspace = setup_workspace_with_insert_recipe("preview_original_modified");
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
    args.insert(
        "file".to_string(),
        serde_json::Value::String(rel.to_string()),
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
    let files = response[protocol_keys::FILES].as_array().unwrap();
    assert!(!files.is_empty());
    assert!(files[0]["original"].as_str().is_some());
    assert!(files[0]["modified"].as_str().is_some());
    assert!(files[0]["patches"].is_null());

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
    args.insert(
        "file".to_string(),
        serde_json::Value::String(rel.to_string()),
    );

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
    assert!(response[protocol_keys::FILE]["original"].as_str().is_some());
    assert!(response[protocol_keys::FILE]["modified"].as_str().is_some());

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn preview_add_log_line_with_class_and_method_args() {
    let repo = repo_root();
    let mut registry =
        RecipeRegistry::new(repo.clone(), repo.join("test/fixtures/dart_demo/.codemod"));
    registry.reload();

    let rel = "test/fixtures/ast_paths/settings.dart";
    let mut args = BTreeMap::new();
    args.insert(
        "file".to_string(),
        serde_json::Value::String(rel.to_string()),
    );
    args.insert(
        "className".to_string(),
        serde_json::Value::String("Settings".to_string()),
    );
    args.insert(
        "methodName".to_string(),
        serde_json::Value::String("update".to_string()),
    );

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
    let files = response[protocol_keys::FILES].as_array().unwrap();
    assert_eq!(files.len(), 1);
    assert!(files[0]["snippet"].as_str().unwrap_or("").contains("print"));
}

#[test]
fn preview_create_file_step() {
    let workspace = temp_workspace("create_step");
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/rust_oracle/scaffold_widget.recipe.luau"),
        recipes_dir.join("scaffold_widget.luau"),
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert(
        "featurePath".to_string(),
        serde_json::Value::String("lib/features".to_string()),
    );
    args.insert(
        "name".to_string(),
        serde_json::Value::String("Counter".to_string()),
    );

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
    let files = response[protocol_keys::FILES].as_array().unwrap();
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
    assert!(std::fs::read_to_string(&created)
        .unwrap()
        .contains("class Counter"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn preview_delete_file_step() {
    let workspace = temp_workspace("delete_step");
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/rust_oracle/delete_legacy.recipe.luau"),
        recipes_dir.join("delete_legacy.luau"),
    )
    .unwrap();

    let legacy = workspace.join("lib/legacy.dart");
    std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
    std::fs::write(&legacy, "class Legacy {}\n").unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert(
        "legacyPath".to_string(),
        serde_json::Value::String("lib/legacy.dart".to_string()),
    );

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
    assert_eq!(preview[protocol_keys::FILES][0]["kind"], "delete");

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

    let inline_recipe = serde_json::json!(
        r#"
return {
  id = "inline_insert",
  args = { { name = "file", required = true } },
  apply = function(c, args)
    c:insert({
      path = args.file,
      language = "dart",
      query = "(class_definition name: (identifier) @className body: (class_body (method_signature (function_signature name: (identifier) @methodName)) (function_body (block) @body)) (#eq? @className \"Settings\") (#eq? @methodName \"update\"))",
      capture = "body",
      anchor = "end",
      text = "    print('inline');\n",
    })
  end,
}
"#
    );

    let mut args = BTreeMap::new();
    args.insert(
        "file".to_string(),
        serde_json::Value::String("lib/settings.dart".to_string()),
    );

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

#[test]
fn content_classification_recognizes_map_in_recipes_directory() {
    let workspace = temp_workspace("cc_map_in_recipes");
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::write(
        recipes_dir.join("my_map.luau"),
        r#"return { id = "my_map", map = { a = "b" } }"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(response["ok"], true);
    let map_ids = response["mapIds"].as_array().unwrap();
    assert!(map_ids.iter().any(|id| id.as_str() == Some("my_map")));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn content_classification_recognizes_recipe_in_maps_directory() {
    let workspace = temp_workspace("cc_recipe_in_maps");
    let maps_dir = workspace.join(".codemod/maps");
    std::fs::create_dir_all(&maps_dir).unwrap();
    std::fs::write(
        maps_dir.join("my_recipe.luau"),
        r#"return { id = "my_recipe", apply = function(c, args) end }"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(response["ok"], true);
    let recipes = response["recipes"].as_array().unwrap();
    assert!(recipes.iter().any(|r| r["id"] == "my_recipe"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn content_classification_recognizes_query_lib_in_types_directory() {
    let workspace = temp_workspace("cc_query_lib_in_types");
    let types_dir = workspace.join(".codemod/types");
    std::fs::create_dir_all(&types_dir).unwrap();
    std::fs::write(
        types_dir.join("my_queries.luau"),
        r#"return { id = "my_queries", queries = { body = "(_) @x" } }"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(response["ok"], true);
    let query_ids = response["queryIds"].as_array().unwrap();
    assert!(query_ids
        .iter()
        .any(|id| id.as_str() == Some("my_queries.body")));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn content_classification_recognizes_type_in_recipes_directory() {
    let workspace = temp_workspace("cc_type_in_recipes");
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::write(
        recipes_dir.join("my_type.luau"),
        r#"return { id = "MyType", fields = { { name = "x" } } }"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(response["ok"], true);
    let type_ids = response["typeIds"].as_array().unwrap();
    assert!(type_ids.iter().any(|id| id.as_str() == Some("MyType")));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn content_classification_recognizes_variables_in_recipes_directory() {
    let workspace = temp_workspace("cc_variables_in_recipes");
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::write(
        recipes_dir.join("my_vars.luau"),
        r#"return { id = "my_vars", values = { a = "b" } }"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(response["ok"], true);
    let var_ids = response["varIds"].as_array().unwrap();
    assert!(var_ids.iter().any(|id| id.as_str() == Some("my_vars")));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn content_classification_skips_d_luau_in_stubs_directory() {
    let workspace = temp_workspace("cc_d_luau_in_stubs");
    let stubs_dir = workspace.join(".codemod/stubs");
    std::fs::create_dir_all(&stubs_dir).unwrap();
    std::fs::write(stubs_dir.join("codemod-context.d.luau"), "return {}").unwrap();
    std::fs::write(
        stubs_dir.join("actual_recipe.luau"),
        r#"return { id = "actual_recipe", apply = function(c, args) end }"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(response["ok"], true);
    let diagnostics = response["diagnostics"]
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    assert!(!diagnostics
        .iter()
        .any(|d| d["file"] == ".codemod/stubs/codemod-context.d.luau"));
    let recipes = response["recipes"].as_array().unwrap();
    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0]["id"], "actual_recipe");

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn content_classification_skips_d_luau_in_types_directory() {
    let workspace = temp_workspace("cc_d_luau_in_types");
    let types_dir = workspace.join(".codemod/types");
    std::fs::create_dir_all(&types_dir).unwrap();
    std::fs::write(types_dir.join("codemod-context.d.luau"), "return {}").unwrap();
    std::fs::write(
        types_dir.join("actual_type.luau"),
        r#"return { id = "ActualType", fields = { { name = "x" } } }"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(response["ok"], true);
    let diagnostics = response["diagnostics"]
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    assert!(!diagnostics
        .iter()
        .any(|d| d["file"] == ".codemod/types/codemod-context.d.luau"));
    let type_ids = response["typeIds"].as_array().unwrap();
    assert!(type_ids.iter().any(|id| id.as_str() == Some("ActualType")));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn content_classification_skips_d_lua_files() {
    let workspace = temp_workspace("cc_d_lua_files");
    let codemod_dir = workspace.join(".codemod");
    std::fs::create_dir_all(&codemod_dir).unwrap();
    std::fs::write(codemod_dir.join("stubs.d.lua"), "return {}").unwrap();
    std::fs::write(
        codemod_dir.join("actual_recipe.luau"),
        r#"return { id = "actual_recipe", apply = function(c, args) end }"#,
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(response["ok"], true);
    let diagnostics = response["diagnostics"]
        .as_array()
        .map(|a| a.as_slice())
        .unwrap_or(&[]);
    assert!(!diagnostics
        .iter()
        .any(|d| d["file"] == ".codemod/stubs.d.lua"));
    let recipes = response["recipes"].as_array().unwrap();
    assert_eq!(recipes.len(), 1);
    assert_eq!(recipes[0]["id"], "actual_recipe");

    let _ = std::fs::remove_dir_all(workspace);
}
