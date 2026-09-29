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
fn flutter_poc_scan_finds_model_and_store_roots_linked_by_relations() {
    let root = fixture_root();
    let mut registry = RecipeRegistry::new(root.join("workspace"), root.join(".codemod"));
    registry.reload();

    let response = scan(&mut registry);
    assert_eq!(response["ok"], true, "scan failed: {response}");

    let instances = response["instances"].as_array().expect("instances");
    assert_eq!(
        instances.len(),
        4,
        "expected 2 model + 2 store root instances"
    );

    let recipe_model = instances
        .iter()
        .find(|i| i["id"] == "flutter.model/recipe")
        .expect("recipe model");
    assert_eq!(recipe_model["summary"]["kind"].as_str(), Some("model"));
    assert_eq!(recipe_model["summary"]["title"].as_str(), Some("Recipe"));

    let children = recipe_model["ast"]["children"].as_array().unwrap();
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

    assert!(
        children.iter().all(|c| c["componentId"] != "flutter.state"),
        "store must be a root linked by relation, not a nested child"
    );

    let summary_fields = recipe_model["summary"]["fields"].as_array().unwrap();
    assert_eq!(summary_fields.len(), 2);
    assert!(summary_fields.iter().any(|v| v == "title"));
    assert!(summary_fields.iter().any(|v| v == "subtitle"));

    let recipe_store = instances
        .iter()
        .find(|i| i["id"] == "flutter.state/recipe")
        .expect("recipe store root");
    assert_eq!(
        recipe_store["ast"]["source"]["path"],
        "lib/stores/recipe_store.dart"
    );
    let store_fields = recipe_store["ast"]["fields"].as_array().unwrap();
    assert_eq!(store_fields.len(), 1);
    assert_eq!(store_fields[0]["name"], "title");

    // Outgoing edges are attached to the store node as relations, grouped by kind.
    let relations = recipe_store["ast"]["relations"]["stores"]
        .as_array()
        .expect("store node relations.stores");
    assert_eq!(relations.len(), 1);
    assert_eq!(relations[0]["to"], "flutter.model/recipe");
    assert_eq!(
        relations[0]["sources"][0]["path"],
        "lib/stores/recipe_store.dart"
    );

    let edges = response["edges"].as_array().expect("edges");
    assert_eq!(edges.len(), 2, "one stores edge per store root");
    assert!(edges.iter().any(|e| e["from"] == "flutter.state/recipe"
        && e["kind"] == "stores"
        && e["to"] == "flutter.model/recipe"));
    assert!(edges.iter().any(|e| e["from"] == "flutter.state/user"
        && e["kind"] == "stores"
        && e["to"] == "flutter.model/user"));

    let kind_graph = response["kindGraph"].as_array().expect("kindGraph");
    assert_eq!(kind_graph.len(), 1);
    assert_eq!(kind_graph[0]["from"], "flutter.state");
    assert_eq!(kind_graph[0]["kind"], "stores");
    assert_eq!(kind_graph[0]["to"], "flutter.model");
    assert_eq!(kind_graph[0]["count"], 2);
}

#[test]
fn flutter_poc_verify_detects_store_drift() {
    let root = fixture_root();
    let mut registry = RecipeRegistry::new(root.join("workspace"), root.join(".codemod"));
    registry.reload();

    let response = scan(&mut registry);
    assert_eq!(response["ok"], true);

    let findings = response["findings"].as_array().expect("findings");
    let codes: Vec<&str> = findings.iter().filter_map(|f| f["code"].as_str()).collect();

    assert!(
        codes.contains(&"model.field_missing_in_store"),
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

    assert!(
        !codes.contains(&"model.missing_store"),
        "both models have stores, got {codes:?}"
    );
}

/// Dogfood: the fixture's own `*.test.luau` suites (component + stack level)
/// must all pass via the testing layer.
#[test]
fn flutter_poc_test_suites_pass() {
    let root = fixture_root();
    let mut registry = RecipeRegistry::new(root.join("workspace"), root.join(".codemod"));
    registry.reload();

    let report = dispatch::handle_command(&mut registry, HostCommand::RunTests { filter: None });
    assert_eq!(report["ok"], true, "test suites failed: {report}");
    assert_eq!(report["totals"]["failed"], 0);
    assert!(
        report["totals"]["passed"].as_u64().unwrap() >= 5,
        "expected component + stack suites to run: {report}"
    );
}
