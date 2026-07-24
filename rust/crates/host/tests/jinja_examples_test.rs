//! Jinja2 / MiniJinja showcase fixtures — casing, maps, conditionals, defaultsTo, inheritance.

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

fn setup_jinja_workspace(name: &str) -> PathBuf {
    let workspace = temp_workspace(name);
    let fixture = repo_root().join("test/fixtures/jinja_examples");
    copy_dir_all(&fixture.join("workspace"), &workspace);
    copy_dir_all(&fixture.join(".codemod"), &workspace.join(".codemod"));
    workspace
}

fn preview_recipe(
    registry: &mut RecipeRegistry,
    recipe_id: &str,
    args: BTreeMap<String, String>,
) -> serde_json::Value {
    dispatch::handle_command(
        registry,
        HostCommand::Preview {
            recipe: Some(recipe_id.to_string()),
            inline_recipe: None,
            args,
            snippet_lines: Some(100),
        },
    )
}

fn file_content(response: &serde_json::Value, path: &str) -> String {
    let file = response["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == path)
        .unwrap_or_else(|| panic!("no preview for {path}"));
    file["modified"]
        .as_str()
        .or_else(|| file["snippet"].as_str())
        .unwrap_or("")
        .to_string()
}

#[test]
fn jinja_examples_validate_all_recipes() {
    let workspace = setup_jinja_workspace("jinja_validate");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let validate = dispatch::handle_command(&mut registry, HostCommand::Validate { recipe: None });
    assert_eq!(validate["ok"], true, "{}", validate["error"]);

    let list = dispatch::handle_command(&mut registry, HostCommand::List);
    let ids: Vec<_> = list["recipes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    for id in [
        "showcase_casing",
        "conditional_create",
        "create_with_layout",
        "defaults_orchestrator",
        "defaults_child",
        "with_bind_child",
        "with_bind_orchestrator",
        "with_partial_orchestrator",
    ] {
        assert!(ids.contains(&id), "missing recipe {id}");
    }

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn with_bindings_forward_and_hardcode() {
    let workspace = setup_jinja_workspace("jinja_with_full");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let schema = registry.get("with_bind_orchestrator").expect("schema");
    let names: Vec<_> = schema.args.iter().map(|a| a.name.as_str()).collect();
    assert!(names.contains(&"featureName"));
    assert!(!names.contains(&"className"));
    assert!(!names.contains(&"suffix"));

    let mut args = BTreeMap::new();
    args.insert("featureName".to_string(), "FeedList".to_string());

    let response = preview_recipe(&mut registry, "with_bind_orchestrator", args);
    assert_eq!(response["ok"], true, "{}", response["error"]);

    let content = file_content(&response, "lib/generated/feed_list_Widget.dart");
    assert!(content.contains("class FeedListWidget {}"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn with_bindings_partial_fallthrough() {
    let workspace = setup_jinja_workspace("jinja_with_partial");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let schema = registry.get("with_partial_orchestrator").expect("schema");
    let names: Vec<_> = schema.args.iter().map(|a| a.name.as_str()).collect();
    assert!(names.contains(&"featureName"));
    assert!(names.contains(&"suffix"));
    assert!(!names.contains(&"className"));

    let mut args = BTreeMap::new();
    args.insert("featureName".to_string(), "Metrics".to_string());
    args.insert("suffix".to_string(), "View".to_string());

    let response = preview_recipe(&mut registry, "with_partial_orchestrator", args);
    assert_eq!(response["ok"], true, "{}", response["error"]);

    let content = file_content(&response, "lib/generated/metrics_View.dart");
    assert!(content.contains("class MetricsView {}"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn showcase_casing_renders_all_filters() {
    let workspace = setup_jinja_workspace("jinja_casing");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("name".to_string(), "FeedList".to_string());
    args.insert("fieldKey".to_string(), "tickCount".to_string());

    let response = preview_recipe(&mut registry, "showcase_casing", args);
    assert_eq!(response["ok"], true, "{}", response["error"]);

    let content = file_content(&response, "lib/generated/feed_list_casing.dart");
    assert!(content.contains("// snake=feed_list"));
    assert!(content.contains("// camel=feedList"));
    assert!(content.contains("// pascal=FeedList"));
    assert!(content.contains("// lower=feedlist"));
    assert!(content.contains("// upper=FEEDLIST"));
    assert!(content.contains("// screaming=FEED_LIST"));
    assert!(content.contains("// kebab=feed-list"));
    assert!(content.contains("// mapFilter=int"));
    assert!(content.contains("// mapContext=int"));
    assert!(content.contains("class FeedListCasing"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn conditional_create_respects_bool_arg() {
    let workspace = setup_jinja_workspace("jinja_conditional");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("className".to_string(), "Counter".to_string());
    args.insert("includeTests".to_string(), "true".to_string());

    let with_tests = preview_recipe(&mut registry, "conditional_create", args.clone());
    assert_eq!(with_tests["ok"], true, "{}", with_tests["error"]);
    let content = file_content(&with_tests, "lib/generated/counter_widget.dart");
    assert!(content.contains("void testHook()"));

    args.insert("includeTests".to_string(), "false".to_string());
    let without_tests = preview_recipe(&mut registry, "conditional_create", args);
    assert_eq!(without_tests["ok"], true, "{}", without_tests["error"]);
    let content = file_content(&without_tests, "lib/generated/counter_widget.dart");
    assert!(!content.contains("void testHook()"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn create_with_layout_uses_extends_and_include() {
    let workspace = setup_jinja_workspace("jinja_layout");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("className".to_string(), "FeedList".to_string());

    let response = preview_recipe(&mut registry, "create_with_layout", args);
    assert_eq!(response["ok"], true, "{}", response["error"]);

    let content = file_content(&response, "lib/generated/feed_list_layout.dart");
    assert!(content.contains("// Generated for FeedList"));
    assert!(content.contains("class FeedListLayoutWidget"));

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn defaults_orchestrator_applies_child_defaults_to() {
    let workspace = setup_jinja_workspace("jinja_defaults");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let mut args = BTreeMap::new();
    args.insert("label".to_string(), "Metrics".to_string());

    let response = preview_recipe(&mut registry, "defaults_orchestrator", args);
    assert_eq!(response["ok"], true, "{}", response["error"]);

    let content = file_content(&response, "lib/generated/metrics_defaults.dart");
    assert!(content.contains("static const verbose = false;"));

    let _ = std::fs::remove_dir_all(workspace);
}
