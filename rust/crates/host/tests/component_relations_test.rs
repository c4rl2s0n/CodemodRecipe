//! Relations v1: relate phase (ref resolution, duplicate merging, unresolved
//! findings, external symbols), graph-aware verify, incremental re-relate, dry-run
//! scan-on-demand, kindGraph aggregation.

mod common;

use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use common::temp_workspace;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const STACK: &str = r#"
return {
	id = "rel-stack",
	components = { "rel.entity", "rel.store" },
}
"#;

/// Entity root: verify intentionally omitted (omission ≡ no judgment).
const ENTITY: &str = r#"
return {
	id = "rel.entity",
	description = "Entity roots (verify intentionally omitted)",
	identify = function(c)
		local out = {}
		for _, row in ipairs(c:matchFiles("src/entities/{name}.dart")) do
			out[#out + 1] = { path = row.path, name = row.name }
		end
		return out
	end,
	parse = function(c, locator)
		return {
			id = locator.name or c:filename(locator.path),
			name = locator.name or c:filename(locator.path),
			path = locator.path,
		}
	end,
}
"#;

/// Store root: relate emits a dedupe-worthy duplicate, a ref, and an external symbol;
/// verify judges with the graph (proves it runs post-relate with resolved edges).
const STORE: &str = r#"
return {
	id = "rel.store",
	description = "Store roots with relate + graph-aware verify",
	identify = function(c)
		local out = {}
		for _, row in ipairs(c:matchFiles("src/stores/{id}_store.dart")) do
			out[#out + 1] = { path = row.path, id = row.id }
		end
		return out
	end,
	parse = function(c, locator)
		local id = locator.id or c:filename(locator.path):gsub("_store$", "")
		return {
			id = id,
			name = c:filename(locator.path),
			path = locator.path,
		}
	end,
	relate = function(_c, instance, _index)
		local ast = instance.ast
		return {
			{
				kind = "stores",
				to = { componentId = "rel.entity", key = ast.id },
				source = { path = ast.path },
			},
			-- duplicate on purpose: host merges by (from, kind, to); identical
			-- provenance yields a relation.duplicate info finding
			{
				kind = "stores",
				to = { componentId = "rel.entity", key = ast.id },
				source = { path = ast.path },
			},
			{ kind = "extends", to = { symbol = "dart:core/Store" } },
		}
	end,
	verify = function(_c, instance, graph)
		local findings = {}
		-- Node-centric traversal: inst:outgoing / inst:follow / inst:neighbors must
		-- behave like the id-based graph calls (the instance arg IS the graph node).
		local stores = instance:outgoing("stores")
		if #stores == 0 then
			findings[#findings + 1] = {
				severity = "warning",
				code = "store.unlinked",
				message = "store has no resolved stores edge",
				instanceId = instance.id,
			}
			return findings
		end
		local target = instance:follow(stores[1])
		if not target or target.componentId ~= "rel.entity" then
			findings[#findings + 1] = {
				severity = "error",
				code = "store.bad_target",
				message = "stores edge did not resolve to an entity",
				instanceId = instance.id,
			}
		end
		if #instance:neighbors("stores") ~= #stores then
			findings[#findings + 1] = {
				severity = "error",
				code = "store.bad_neighbors",
				message = "neighbors() disagrees with outgoing()",
				instanceId = instance.id,
			}
		end
		return findings
	end,
	scaffold = {
		args = {},
		apply = function(_c, _args, graph)
			-- scaffold.apply receives the pre-scaffold graph as 3rd arg.
			assert(graph ~= nil and type(graph.nodes) == "function", "scaffold graph missing")
			assert(type(graph:nodes()) == "table", "graph:nodes() must return a table")
		end,
	},
}
"#;

fn setup_relations_workspace(name: &str) -> PathBuf {
    let root = temp_workspace(name);
    let _ = fs::remove_dir_all(&root); // clean slate (temp dirs are pid-scoped)
    let workspace = root.join("workspace");
    let components = root.join(".codemod").join("components");
    fs::create_dir_all(workspace.join("src/entities")).unwrap();
    fs::create_dir_all(workspace.join("src/stores")).unwrap();
    fs::create_dir_all(&components).unwrap();

    fs::write(root.join(".codemod/stack.luau"), STACK).unwrap();
    fs::write(components.join("rel.entity.luau"), ENTITY).unwrap();
    fs::write(components.join("rel.store.luau"), STORE).unwrap();

    fs::write(
        workspace.join("src/entities/recipe.dart"),
        "class Recipe {\n  final String title;\n}\n",
    )
    .unwrap();
    fs::write(
        workspace.join("src/stores/recipe_store.dart"),
        "class RecipeStore {}\n",
    )
    .unwrap();
    // No src/entities/ghost.dart: the ghost store's ref is unresolved drift.
    fs::write(
        workspace.join("src/stores/ghost_store.dart"),
        "class GhostStore {}\n",
    )
    .unwrap();

    root
}

fn registry_for(root: &Path) -> RecipeRegistry {
    let mut registry = RecipeRegistry::new(root.join("workspace"), root.join(".codemod"));
    registry.reload();
    registry
}

fn scan(registry: &mut RecipeRegistry) -> serde_json::Value {
    dispatch::handle_command(registry, HostCommand::ComponentScan)
}

fn finding_codes(response: &serde_json::Value) -> Vec<&str> {
    response["findings"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|f| f["code"].as_str()).collect())
        .unwrap_or_default()
}

#[test]
fn relate_phase_resolves_refs_merges_duplicates_and_flags_unresolved() {
    let root = setup_relations_workspace("rel_scan");
    let mut registry = registry_for(&root);

    let response = scan(&mut registry);
    assert_eq!(response["ok"], true, "scan failed: {response}");

    let instances = response["instances"].as_array().expect("instances");
    assert_eq!(instances.len(), 3, "entity + 2 store roots");

    let edges = response["edges"].as_array().expect("edges");
    // recipe stores (merged to 1) + 2 external extends; ghost stores dropped.
    assert_eq!(edges.len(), 3, "edges: {edges:?}");
    assert!(edges.iter().any(|e| e["from"] == "rel.store/recipe"
        && e["kind"] == "stores"
        && e["to"] == "rel.entity/recipe"
        && e["sources"][0]["path"] == "src/stores/recipe_store.dart"));

    // The fixture's identical-provenance duplicate merges without growing sources
    // and yields one relation.duplicate info finding.
    let stores_edge = edges
        .iter()
        .find(|e| e["from"] == "rel.store/recipe" && e["kind"] == "stores")
        .expect("merged stores edge");
    assert_eq!(
        stores_edge["sources"].as_array().map(|s| s.len()),
        Some(1),
        "identical provenance is not double-counted: {stores_edge}"
    );
    let duplicates: Vec<_> = response["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .filter(|f| f["code"] == "relation.duplicate")
        .collect();
    assert_eq!(duplicates.len(), 1, "findings: {:?}", response["findings"]);
    assert_eq!(duplicates[0]["severity"], "info");
    assert_eq!(duplicates[0]["instanceId"], "rel.store/recipe");
    let extends: Vec<_> = edges.iter().filter(|e| e["kind"] == "extends").collect();
    assert_eq!(extends.len(), 2);
    assert!(extends
        .iter()
        .all(|e| e["symbol"] == "dart:core/Store" && e.get("to").is_none()));

    // Unresolved ref → drift finding, edge dropped.
    let findings = response["findings"].as_array().expect("findings");
    let unresolved: Vec<_> = findings
        .iter()
        .filter(|f| f["code"] == "relation.unresolved")
        .collect();
    assert_eq!(unresolved.len(), 1, "findings: {findings:?}");
    assert_eq!(unresolved[0]["instanceId"], "rel.store/ghost");
    assert!(
        unresolved[0]["message"]
            .as_str()
            .unwrap_or("")
            .contains("rel.entity/ghost"),
        "message should name the dangling ref: {}",
        unresolved[0]["message"]
    );

    // Outgoing edges attached to nodes, grouped by kind.
    let recipe_store = instances
        .iter()
        .find(|i| i["id"] == "rel.store/recipe")
        .expect("recipe store");
    let relations = recipe_store["ast"]["relations"]
        .as_object()
        .expect("relations");
    assert_eq!(relations["stores"][0]["to"], "rel.entity/recipe");
    assert_eq!(relations["extends"][0]["symbol"], "dart:core/Store");
}

#[test]
fn verify_runs_post_relate_with_graph_and_omitted_verify_is_quiet() {
    let root = setup_relations_workspace("rel_verify");
    let mut registry = registry_for(&root);

    let response = scan(&mut registry);
    assert_eq!(response["ok"], true, "scan failed: {response}");
    let findings = response["findings"].as_array().expect("findings");

    // Ghost store's stores edge was dropped at resolution, so graph:outgoing is
    // empty in verify → store.unlinked. Proves verify saw the resolved graph.
    let unlinked: Vec<_> = findings
        .iter()
        .filter(|f| f["code"] == "store.unlinked")
        .collect();
    assert_eq!(unlinked.len(), 1);
    assert_eq!(unlinked[0]["instanceId"], "rel.store/ghost");

    // Resolved store: no judgment findings (the fixture's intentional duplicate
    // still reports relation.duplicate — that is parse-phase information, not
    // verify judgment).
    assert!(
        findings
            .iter()
            .all(|f| f["instanceId"] != "rel.store/recipe" || f["code"] == "relation.duplicate"),
        "recipe store should be clean: {findings:?}"
    );

    // rel.entity omits verify → no judgment findings from entity instances.
    assert!(
        findings.iter().all(|f| {
            f["instanceId"]
                .as_str()
                .map(|id| !id.starts_with("rel.entity/"))
                .unwrap_or(true)
        }),
        "omitted verify must be quiet: {findings:?}"
    );
}

#[test]
fn kind_graph_aggregates_instance_and_symbol_edges() {
    let root = setup_relations_workspace("rel_kindgraph");
    let mut registry = registry_for(&root);

    let response = scan(&mut registry);
    assert_eq!(response["ok"], true, "scan failed: {response}");

    let kind_graph = response["kindGraph"].as_array().expect("kindGraph");
    assert_eq!(kind_graph.len(), 2, "kindGraph: {kind_graph:?}");

    let stores = kind_graph
        .iter()
        .find(|k| k["kind"] == "stores")
        .expect("stores kind edge");
    assert_eq!(stores["from"], "rel.store");
    assert_eq!(stores["to"], "rel.entity");
    assert_eq!(stores["count"], 1);
    assert_ne!(stores["external"], true);

    let extends = kind_graph
        .iter()
        .find(|k| k["kind"] == "extends")
        .expect("extends kind edge");
    assert_eq!(extends["from"], "rel.store");
    assert_eq!(extends["to"], "dart:core/Store");
    assert_eq!(extends["count"], 2);
    assert_eq!(extends["external"], true);
}

#[test]
fn dry_run_relate_and_verify_trigger_scan_when_uncached() {
    let root = setup_relations_workspace("rel_dry_run");
    let mut registry = registry_for(&root);

    // No component_scan call: the first facade dry-run must trigger a full scan.
    let instance = serde_json::json!({
        "id": "rel.store/recipe",
        "componentId": "rel.store",
        "locator": { "path": "src/stores/recipe_store.dart", "id": "recipe" },
        "ast": { "id": "recipe", "name": "recipe_store", "path": "src/stores/recipe_store.dart" },
    });

    let relate = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentInvokeHook {
            component_id: "rel.store".to_string(),
            hook: "relate".to_string(),
            preview: true,
            locator: None,
            instance: Some(instance.clone()),
            operation: None,
            args: BTreeMap::new(),
                    preview_token: None,
        },
    );
    assert_eq!(relate["ok"], true, "relate dry-run failed: {relate}");
    let raw_edges = relate["result"].as_array().expect("relate result array");
    assert_eq!(
        raw_edges.len(),
        3,
        "raw hook return (pre-merge): {raw_edges:?}"
    );

    // Verify dry-run reuses the now-populated cache; the recipe store's edge
    // resolved during that scan, so graph:outgoing is non-empty → no findings.
    let verify = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentInvokeHook {
            component_id: "rel.store".to_string(),
            hook: "verify".to_string(),
            preview: true,
            locator: None,
            instance: Some(instance),
            operation: None,
            args: BTreeMap::new(),
                    preview_token: None,
        },
    );
    assert_eq!(verify["ok"], true, "verify dry-run failed: {verify}");
    // Empty Lua arrays serialize as objects; non-empty as arrays.
    if let Some(arr) = verify["result"].as_array() {
        assert!(arr.is_empty(), "expected no findings, got {arr:?}");
    }

    // Omitted verify dry-run ≡ no judgment.
    let entity_verify = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentInvokeHook {
            component_id: "rel.entity".to_string(),
            hook: "verify".to_string(),
            preview: true,
            locator: None,
            instance: Some(serde_json::json!({
                "id": "rel.entity/recipe",
                "componentId": "rel.entity",
                "ast": { "id": "recipe" },
            })),
            operation: None,
            args: BTreeMap::new(),
        },
    );
    assert_eq!(entity_verify["ok"], true, "{entity_verify}");
    if let Some(arr) = entity_verify["result"].as_array() {
        assert!(arr.is_empty(), "omitted verify must return [], got {arr:?}");
    }
}

#[test]
fn incremental_rescan_re_relates_merged_forest() {
    let root = setup_relations_workspace("rel_incremental");
    let mut registry = registry_for(&root);

    let initial = scan(&mut registry);
    assert_eq!(initial["ok"], true);
    assert!(finding_codes(&initial).contains(&"relation.unresolved"));
    assert!(finding_codes(&initial).contains(&"store.unlinked"));

    // The ghost entity appears: the ghost store's ref must now resolve, even
    // though the store file itself is untouched (relate re-runs over the
    // merged forest).
    fs::write(
        root.join("workspace/src/entities/ghost.dart"),
        "class Ghost {\n  final String id;\n}\n",
    )
    .unwrap();

    let updated = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentIncrementalScan {
            dirty_paths: Vec::new(),
            detect_changes: true,
            force: false,
        },
    );
    assert_eq!(updated["ok"], true, "incremental failed: {updated}");

    let codes = finding_codes(&updated);
    assert!(
        !codes.contains(&"relation.unresolved"),
        "ref should resolve after entity appears: {codes:?}"
    );
    assert!(
        !codes.contains(&"store.unlinked"),
        "ghost store should be linked: {codes:?}"
    );

    let edges = updated["edges"].as_array().expect("edges");
    let stores: Vec<_> = edges.iter().filter(|e| e["kind"] == "stores").collect();
    assert_eq!(stores.len(), 2, "both stores now resolve: {edges:?}");
    assert!(stores
        .iter()
        .any(|e| e["from"] == "rel.store/ghost" && e["to"] == "rel.entity/ghost"));

    let kind_graph = updated["kindGraph"].as_array().expect("kindGraph");
    let stores_kind = kind_graph
        .iter()
        .find(|k| k["kind"] == "stores")
        .expect("stores kind edge");
    assert_eq!(stores_kind["count"], 2);
}

#[test]
fn component_graph_renders_kind_and_instance_mermaid() {
    let root = setup_relations_workspace("rel_graph");
    let mut registry = registry_for(&root);

    // Kind-level on an uncached registry: ComponentGraph must scan on demand.
    let kinds = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentGraph {
            level: None,
            out: None,
        },
    );
    assert_eq!(kinds["ok"], true, "kinds failed: {kinds}");
    let mermaid = kinds["mermaid"].as_str().expect("mermaid");
    assert!(mermaid.starts_with("graph LR\n"), "header: {mermaid}");
    assert!(
        mermaid.contains("rel_store[\"rel.store\"] -- \"stores\" --> rel_entity[\"rel.entity\"]"),
        "stores kind edge (×1 omitted): {mermaid}"
    );
    assert!(
        mermaid.contains(
            "rel_store[\"rel.store\"] -- \"extends ×2\" --> dart_core_Store[\"dart:core/Store (external)\"]"
        ),
        "external extends roll-up: {mermaid}"
    );
    assert!(
        !mermaid.contains("ghost"),
        "unresolved ghost ref must not appear at kind level: {mermaid}"
    );
    assert!(kinds["edges"].is_null(), "kind level omits instance edges");
    assert!(
        kinds["kindGraph"].as_array().is_some_and(|k| !k.is_empty()),
        "kindGraph payload present: {kinds}"
    );

    // Instance-level: every flat node, symbol targets as external leaves.
    let instances = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentGraph {
            level: Some("instances".to_string()),
            out: None,
        },
    );
    assert_eq!(instances["ok"], true, "instances failed: {instances}");
    let mermaid = instances["mermaid"].as_str().expect("mermaid");
    assert!(
        mermaid.contains("rel_store_recipe[\"rel.store/recipe\"]"),
        "instance node: {mermaid}"
    );
    assert!(
        mermaid.contains("rel_store_recipe -- \"stores\" --> rel_entity_recipe"),
        "instance edge: {mermaid}"
    );
    assert!(
        mermaid.contains("dart_core_Store[\"dart:core/Store (external)\"]"),
        "symbol leaf: {mermaid}"
    );
    assert!(
        mermaid.contains("rel_store_ghost[\"rel.store/ghost\"]"),
        "isolated node still listed: {mermaid}"
    );
    assert!(
        instances["edges"].as_array().is_some_and(|e| !e.is_empty()),
        "instance level includes edges payload: {instances}"
    );

    // Unknown level → error response.
    let bad = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentGraph {
            level: Some("bogus".to_string()),
            out: None,
        },
    );
    assert_eq!(bad["ok"], false, "bogus level must fail: {bad}");
}

#[test]
fn scaffold_receives_graph_as_third_arg() {
    let root = setup_relations_workspace("rel_scaffold_graph");
    let mut registry = registry_for(&root);

    // Dry-run: the fixture scaffold asserts the graph facade arrived (3rd arg).
    let preview = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentInvokeHook {
            component_id: "rel.store".to_string(),
            hook: "scaffold".to_string(),
            preview: true,
            locator: None,
            instance: None,
            operation: None,
            args: BTreeMap::new(),
                    preview_token: None,
        },
    );
    assert_eq!(preview["ok"], true, "scaffold dry-run failed: {preview}");
}
