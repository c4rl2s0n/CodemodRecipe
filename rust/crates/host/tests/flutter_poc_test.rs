use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../test/fixtures/flutter_poc")
}

fn scan(registry: &mut RecipeRegistry) -> serde_json::Value {
    dispatch::handle_command(registry, HostCommand::ComponentScan)
}

#[test]
fn flutter_poc_scan_finds_models_with_nested_children_and_summaries() {
    let root = fixture_root();
    let mut registry = RecipeRegistry::new(root.join("workspace"), root.join(".codemod"));
    registry.reload();

    let response = scan(&mut registry);
    assert_eq!(response["ok"], true, "scan failed: {response}");

    let instances = response["instances"].as_array().expect("instances");
    assert_eq!(instances.len(), 2, "expected 2 model root instances");

    let recipe_model = instances
        .iter()
        .find(|i| i["id"] == "flutter.model/recipe")
        .expect("recipe model");
    assert_eq!(recipe_model["summary"]["kind"].as_str(), Some("model"));
    assert_eq!(recipe_model["summary"]["title"].as_str(), Some("Recipe"));

    let children = recipe_model["parsed"]["children"].as_array().unwrap();
    let properties: Vec<_> = children
        .iter()
        .filter(|c| c["componentId"] == "flutter.property")
        .collect();
    assert_eq!(properties.len(), 2);
    assert_eq!(properties[0]["name"], "title");
    assert_eq!(properties[1]["name"], "subtitle");

    let methods: Vec<_> = children
        .iter()
        .filter(|c| c["componentId"] == "flutter.method")
        .collect();
    assert_eq!(methods.len(), 1);
    assert_eq!(methods[0]["name"], "copy");

    let store = children
        .iter()
        .find(|c| c["componentId"] == "flutter.state")
        .expect("nested store child");
    assert_eq!(store["source"]["path"], "lib/stores/recipe_store.dart");
    let store_fields = store["fields"].as_array().unwrap();
    assert_eq!(store_fields.len(), 1);
    assert_eq!(store_fields[0]["name"], "title");

    let summary_fields = recipe_model["summary"]["fields"].as_array().unwrap();
    assert_eq!(summary_fields.len(), 2);
    assert!(summary_fields.iter().any(|v| v == "title"));
    assert!(summary_fields.iter().any(|v| v == "subtitle"));
}

#[test]
fn flutter_poc_verify_detects_store_drift() {
    let root = fixture_root();
    let mut registry = RecipeRegistry::new(root.join("workspace"), root.join(".codemod"));
    registry.reload();

    let response = scan(&mut registry);
    assert_eq!(response["ok"], true);

    let findings = response["findings"].as_array().expect("findings");
    let codes: Vec<&str> = findings
        .iter()
        .filter_map(|f| f["code"].as_str())
        .collect();

    assert!(
        codes.iter().any(|c| *c == "model.field_missing_in_store"),
        "expected model field missing in store finding, got {codes:?} full={findings:?}"
    );

    let recipe_model_findings: Vec<_> = findings
        .iter()
        .filter(|f| f["instanceId"] == "flutter.model/recipe")
        .collect();
    assert!(
        recipe_model_findings
            .iter()
            .any(|f| f["code"] == "model.field_missing_in_store"),
        "recipe model should flag subtitle missing from store"
    );
}
