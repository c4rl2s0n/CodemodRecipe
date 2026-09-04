use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../test/fixtures/reference_stack")
}

#[test]
fn component_scan_finds_root_instances_and_child_properties() {
    let root = fixture_root();
    let workspace = root.join("workspace");
    let codemod = root.join(".codemod");
    let mut registry = RecipeRegistry::new(workspace, codemod);
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::ComponentScan);
    assert_eq!(response["ok"], true, "scan failed: {response}");

    let instances = response["instances"].as_array().expect("instances array");
    assert_eq!(instances.len(), 2, "expected entity + store instances");

    let entity = instances
        .iter()
        .find(|i| i["componentId"] == "example.entity")
        .expect("entity instance");
    assert_eq!(entity["id"], "example.entity/recipe");
    let fields = entity["parsed"]["fields"].as_array().expect("fields");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0]["name"], "title");

    let store = instances
        .iter()
        .find(|i| i["componentId"] == "example.store")
        .expect("store instance");
    assert_eq!(store["id"], "example.store/recipe");
}

#[test]
fn component_invoke_operation_preview_and_apply() {
    let root = fixture_root();
    let workspace = root.join("workspace");
    let codemod = root.join(".codemod");
    let entity_path = workspace.join("src/entities/recipe.dart");
    let original = std::fs::read_to_string(&entity_path).unwrap();

    let mut registry = RecipeRegistry::new(workspace.clone(), codemod);
    registry.reload();

    let preview = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentInvokeOperation {
            instance_id: "example.entity/recipe".to_string(),
            operation: "addField".to_string(),
            args: [("name".to_string(), serde_json::json!("subtitle"))]
                .into_iter()
                .collect(),
            preview: true,
        },
    );
    assert_eq!(preview["ok"], true, "preview failed: {preview}");
    let preview_src = std::fs::read_to_string(&entity_path).unwrap();
    assert_eq!(preview_src, original, "preview must not write disk");

    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentInvokeOperation {
            instance_id: "example.entity/recipe".to_string(),
            operation: "addField".to_string(),
            args: [("name".to_string(), serde_json::json!("subtitle"))]
                .into_iter()
                .collect(),
            preview: false,
        },
    );
    assert_eq!(apply["ok"], true, "apply failed: {apply}");
    let applied_src = std::fs::read_to_string(&entity_path).unwrap();
    assert!(
        applied_src.contains("final String subtitle"),
        "expected new field in source"
    );
    let rescanned = apply["instance"]["parsed"]["fields"]
        .as_array()
        .expect("rescanned fields");
    assert_eq!(rescanned.len(), 2);

    std::fs::write(&entity_path, original).unwrap();
}
