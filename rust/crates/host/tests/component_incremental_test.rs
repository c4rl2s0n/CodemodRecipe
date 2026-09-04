mod common;

use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use common::{copy_dir_all, repo_root, temp_workspace};
use std::fs;
use std::path::{Path, PathBuf};

fn setup_flutter_poc(name: &str) -> PathBuf {
    let workspace = temp_workspace(name);
    let fixture = repo_root().join("test/fixtures/flutter_poc");
    copy_dir_all(&fixture.join("workspace"), &workspace);
    copy_dir_all(&fixture.join(".codemod"), &workspace.join(".codemod"));
    workspace
}

fn registry_for(workspace: &Path) -> RecipeRegistry {
    let mut registry = RecipeRegistry::new(workspace.to_path_buf(), workspace.join(".codemod"));
    registry.reload();
    registry
}

fn scan(registry: &mut RecipeRegistry) -> serde_json::Value {
    dispatch::handle_command(registry, HostCommand::ComponentScan)
}

fn incremental(
    registry: &mut RecipeRegistry,
    dirty_paths: Vec<&str>,
    detect_changes: bool,
) -> serde_json::Value {
    dispatch::handle_command(
        registry,
        HostCommand::ComponentIncrementalScan {
            dirty_paths: dirty_paths.into_iter().map(String::from).collect(),
            detect_changes,
            force: false,
        },
    )
}

#[test]
fn incremental_dirty_paths_updates_store_drift() {
    let workspace = setup_flutter_poc("incr_dirty_paths");
    let store_path = workspace.join("lib/stores/recipe_store.dart");
    let original = fs::read_to_string(&store_path).unwrap();

    let mut registry = registry_for(&workspace);
    let initial = scan(&mut registry);
    assert_eq!(initial["ok"], true);
    let initial_codes: Vec<&str> = initial["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["code"].as_str())
        .collect();
    assert!(initial_codes.contains(&"model.field_missing_in_store"));

    let patched = original.replace(
        "String title = '';",
        "String title = '';\n  String subtitle = '';",
    );
    fs::write(&store_path, &patched).unwrap();

    let updated = incremental(&mut registry, vec!["lib/stores/recipe_store.dart"], false);
    assert_eq!(updated["ok"], true, "incremental failed: {updated}");
    let codes: Vec<&str> = updated["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["code"].as_str())
        .collect();
    assert!(
        !codes.contains(&"model.field_missing_in_store"),
        "expected subtitle drift cleared, got {codes:?}"
    );

    let recipe = updated["instances"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == "flutter.model/recipe")
        .expect("recipe instance");
    let store_child = recipe["node"]["children"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["componentId"] == "flutter.state")
        .expect("store child");
    assert_eq!(store_child["fields"].as_array().unwrap().len(), 2);
}

#[test]
fn incremental_detect_changes_after_store_edit() {
    let workspace = setup_flutter_poc("incr_detect_edit");
    let store_path = workspace.join("lib/stores/recipe_store.dart");
    let original = fs::read_to_string(&store_path).unwrap();

    let mut registry = registry_for(&workspace);
    assert_eq!(scan(&mut registry)["ok"], true);

    fs::write(
        &store_path,
        original.replace(
            "String title = '';",
            "String title = '';\n  String subtitle = '';",
        ),
    )
    .unwrap();

    let updated = incremental(&mut registry, vec![], true);
    assert_eq!(updated["ok"], true, "detect failed: {updated}");
    let codes: Vec<&str> = updated["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|f| f["code"].as_str())
        .collect();
    assert!(!codes.contains(&"model.field_missing_in_store"));
}

#[test]
fn incremental_detect_changes_adds_new_model_file() {
    let workspace = setup_flutter_poc("incr_detect_new");
    let model_path = workspace.join("lib/models/order.dart");
    let store_path = workspace.join("lib/stores/order_store.dart");

    let mut registry = registry_for(&workspace);
    let initial = scan(&mut registry);
    assert_eq!(initial["instances"].as_array().unwrap().len(), 2);

    fs::write(
        &model_path,
        "class Order {\n  final String id;\n  Order({required this.id});\n}\n",
    )
    .unwrap();
    fs::write(&store_path, "class OrderStore {\n  String id = '';\n}\n").unwrap();

    let updated = incremental(&mut registry, vec![], true);
    assert_eq!(updated["ok"], true, "detect new file failed: {updated}");
    assert_eq!(updated["instances"].as_array().unwrap().len(), 3);
    assert!(updated["instances"]
        .as_array()
        .unwrap()
        .iter()
        .any(|i| i["id"] == "flutter.model/order"));
}

#[test]
fn incremental_noop_when_nothing_changed() {
    let workspace = setup_flutter_poc("incr_noop");
    let mut registry = registry_for(&workspace);
    let initial = scan(&mut registry);
    assert_eq!(initial["ok"], true);

    let again = incremental(&mut registry, vec![], true);
    assert_eq!(again["ok"], true);
    assert_eq!(
        again["instances"].as_array().unwrap().len(),
        initial["instances"].as_array().unwrap().len()
    );
}

#[test]
fn invoke_add_store_field_clears_drift_via_incremental_rescan() {
    let workspace = setup_flutter_poc("incr_op_rescan");
    let store_path = workspace.join("lib/stores/recipe_store.dart");

    let mut registry = registry_for(&workspace);
    scan(&mut registry);

    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentInvokeOperation {
            instance_id: "flutter.model/recipe".to_string(),
            operation: "addStoreField".to_string(),
            args: [("fieldName".to_string(), serde_json::json!("subtitle"))]
                .into_iter()
                .collect(),
            preview: false,
        },
    );
    assert_eq!(apply["ok"], true, "apply failed: {apply}");

    let findings = apply["findings"].as_array();
    let codes: Vec<&str> = findings
        .map(|arr| {
            arr.iter()
                .filter_map(|f| f["code"].as_str())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert!(
        !codes.contains(&"model.field_missing_in_store"),
        "expected drift cleared after addStoreField, got {codes:?}"
    );

    let applied_src = fs::read_to_string(&store_path).unwrap();
    assert!(applied_src.contains("subtitle"));
}

#[test]
fn component_scaffold_creates_model_and_store() {
    let workspace = setup_flutter_poc("incr_scaffold");
    let mut registry = registry_for(&workspace);
    scan(&mut registry);

    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentScaffold {
            component_id: "flutter.model".to_string(),
            args: [("name".to_string(), serde_json::json!("Order"))]
                .into_iter()
                .collect(),
            preview: false,
        },
    );
    assert_eq!(apply["ok"], true, "scaffold failed: {apply}");

    assert!(workspace.join("lib/models/order.dart").is_file());
    assert!(workspace.join("lib/stores/order_store.dart").is_file());

    let again = scan(&mut registry);
    assert_eq!(again["ok"], true, "rescan failed: {again}");
    assert!(
        again["instances"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == "flutter.model/order"),
        "expected order instance after scaffold"
    );
}
