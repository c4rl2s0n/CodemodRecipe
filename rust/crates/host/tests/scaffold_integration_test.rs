//! End-to-end scaffold: compose, create (templateFile), multi-file edit, maps, delete.

use codemod_recipe_host::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::registry::RecipeRegistry;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static WORKSPACE_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn temp_workspace(name: &str) -> PathBuf {
    let n = WORKSPACE_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("{name}_{}_{n}", std::process::id()))
}

fn copy_dir_all(src: &Path, dst: &Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let file_type = entry.file_type().unwrap();
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).unwrap();
        }
    }
}

fn setup_scaffold_workspace(name: &str) -> PathBuf {
    let workspace = temp_workspace(name);
    let fixture = repo_root().join("test/fixtures/scaffold_project");
    copy_dir_all(&fixture.join("workspace"), &workspace);
    copy_dir_all(&fixture.join(".codemod"), &workspace.join(".codemod"));
    workspace
}

fn scaffold_args() -> BTreeMap<String, String> {
    let mut args = BTreeMap::new();
    args.insert("className".to_string(), "Counter".to_string());
    args.insert("fieldName".to_string(), "tickCount".to_string());
    args
}

#[test]
fn scaffold_project_validate_describe_and_catalog() {
    let workspace = setup_scaffold_workspace("scaffold_validate");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let validate = dispatch::handle_command(&mut registry, HostCommand::Validate { recipe: None });
    assert_eq!(validate["ok"], true, "{}", validate["error"]);
    let diagnostics = validate["diagnostics"].as_array();
    if let Some(items) = diagnostics {
        assert!(
            items.iter().all(|d| d["severity"] != "error"),
            "unexpected errors: {items:?}"
        );
    }

    let list = dispatch::handle_command(&mut registry, HostCommand::List);
    assert_eq!(list["ok"], true);
    let ids: Vec<_> = list["recipes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"scaffold_feature"));
    assert!(ids.contains(&"create_repository"));
    assert!(ids.contains(&"patch_counter"));
    assert!(ids.contains(&"patch_app"));
    assert!(list["mapsLoaded"].as_u64().unwrap_or(0) >= 1);

    let describe = dispatch::handle_command(
        &mut registry,
        HostCommand::Describe {
            recipe: "scaffold_feature".to_string(),
        },
    );
    assert_eq!(describe["ok"], true);
    let recipe = &describe["recipe"];
    assert_eq!(recipe["id"], "scaffold_feature");
    assert!(recipe["args"].as_array().unwrap().len() >= 2);

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn scaffold_project_preview_covers_create_edit_and_delete() {
    let workspace = setup_scaffold_workspace("scaffold_preview");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("scaffold_feature".to_string()),
            inline_recipe: None,
            args: scaffold_args(),
            snippet_lines: Some(3),
        },
    );
    assert_eq!(response["ok"], true, "{}", response["error"]);

    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 4, "expected create + 2 edits + delete");

    let kinds: Vec<_> = files.iter().map(|f| f["kind"].as_str().unwrap()).collect();
    assert_eq!(kinds.iter().filter(|k| **k == "create").count(), 1);
    assert_eq!(kinds.iter().filter(|k| **k == "edit").count(), 2);
    assert_eq!(kinds.iter().filter(|k| **k == "delete").count(), 1);

    let create = files.iter().find(|f| f["kind"] == "create").unwrap();
    assert_eq!(create["path"], "lib/counter/counter_repository.dart");
    assert!(create["isNew"].as_bool().unwrap());
    let create_snippet = create["snippet"].as_str().unwrap_or("");
    assert!(create_snippet.contains("CounterRepository"));

    let counter_preview = files
        .iter()
        .find(|f| f["path"] == "lib/counter/counter.dart")
        .unwrap();
    assert!(!counter_preview["patches"].as_array().unwrap().is_empty());
    assert!(counter_preview["snippet"]
        .as_str()
        .unwrap_or("")
        .contains("tickCount"));

    let app_preview = files.iter().find(|f| f["path"] == "lib/app.dart").unwrap();
    assert!(app_preview["snippet"]
        .as_str()
        .unwrap_or("")
        .contains("scaffold"));

    let delete_preview = files
        .iter()
        .find(|f| f["path"] == "lib/legacy/stale.dart")
        .unwrap();
    assert_eq!(delete_preview["kind"], "delete");
    assert!(!delete_preview["isNew"].as_bool().unwrap());

    assert!(response["previewToken"]
        .as_str()
        .is_some_and(|t| !t.is_empty()));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn scaffold_project_apply_transforms_workspace() {
    let workspace = setup_scaffold_workspace("scaffold_apply");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let args = scaffold_args();
    let preview = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("scaffold_feature".to_string()),
            inline_recipe: None,
            args: args.clone(),
            snippet_lines: None,
        },
    );
    assert_eq!(preview["ok"], true, "{}", preview["error"]);
    let token = preview["previewToken"].as_str().unwrap();

    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::Apply {
            recipe: Some("scaffold_feature".to_string()),
            inline_recipe: None,
            args,
            preview_token: token.to_string(),
            selection: serde_json::json!({}),
        },
    );
    assert_eq!(apply["ok"], true, "{}", apply["error"]);
    assert_eq!(apply["applied"].as_array().unwrap().len(), 4);

    let repo_path = workspace.join("lib/counter/counter_repository.dart");
    assert!(repo_path.exists());
    let repo_src = std::fs::read_to_string(&repo_path).unwrap();
    assert!(repo_src.contains("class CounterRepository"));

    let counter_src = std::fs::read_to_string(workspace.join("lib/counter/counter.dart")).unwrap();
    assert!(counter_src.contains("final int tickCount = 0;"));

    let app_src = std::fs::read_to_string(workspace.join("lib/app.dart")).unwrap();
    assert!(app_src.contains("print('scaffold')"));
    assert!(app_src.contains("print('starting')"));

    let legacy = workspace.join("lib/legacy/stale.dart");
    assert!(!legacy.exists());

    // Re-preview after apply: create fails (ifExists: fail), proving token/snapshot path.
    let replay = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("scaffold_feature".to_string()),
            inline_recipe: None,
            args: scaffold_args(),
            snippet_lines: None,
        },
    );
    assert_eq!(replay["ok"], false);
    assert!(replay["error"]
        .as_str()
        .unwrap_or("")
        .contains("already exists"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn scaffold_project_delete_if_missing_skip_is_idempotent() {
    let workspace = setup_scaffold_workspace("scaffold_delete_skip");
    let legacy = workspace.join("lib/legacy/stale.dart");
    std::fs::remove_file(&legacy).unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!({
        "id": "delete_only",
        "steps": [{
            "delete": {
                "path": "lib/legacy/stale.dart",
                "ifMissing": "skip"
            }
        }]
    });

    let preview = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: None,
            inline_recipe: Some(inline.clone()),
            args: BTreeMap::new(),
            snippet_lines: None,
        },
    );
    assert_eq!(preview["ok"], true, "{}", preview["error"]);
    let files = preview["files"].as_array().unwrap();
    assert!(
        files.is_empty(),
        "skipped delete should not appear in preview"
    );

    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::Apply {
            recipe: None,
            inline_recipe: Some(inline),
            args: BTreeMap::new(),
            preview_token: preview["previewToken"].as_str().unwrap().to_string(),
            selection: serde_json::json!({}),
        },
    );
    assert_eq!(apply["ok"], true);
    assert_eq!(apply["applied"].as_array().unwrap().len(), 0);

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn scaffold_project_apply_with_patch_selection() {
    let workspace = setup_scaffold_workspace("scaffold_selection");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let args = scaffold_args();
    let preview = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: Some("scaffold_feature".to_string()),
            inline_recipe: None,
            args: args.clone(),
            snippet_lines: None,
        },
    );
    assert_eq!(preview["ok"], true, "{}", preview["error"]);

    // Apply everything except the app.dart edit.
    let selection = serde_json::json!({
        "files": {
            "lib/app.dart": { "include": false }
        }
    });

    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::Apply {
            recipe: Some("scaffold_feature".to_string()),
            inline_recipe: None,
            args,
            preview_token: preview["previewToken"].as_str().unwrap().to_string(),
            selection,
        },
    );
    assert_eq!(apply["ok"], true, "{}", apply["error"]);

    assert!(workspace
        .join("lib/counter/counter_repository.dart")
        .exists());
    assert!(
        std::fs::read_to_string(workspace.join("lib/counter/counter.dart"))
            .unwrap()
            .contains("tickCount")
    );
    assert!(!std::fs::read_to_string(workspace.join("lib/app.dart"))
        .unwrap()
        .contains("print('scaffold')"));
    assert!(!workspace.join("lib/legacy/stale.dart").exists());

    let _ = std::fs::remove_dir_all(workspace);
}
