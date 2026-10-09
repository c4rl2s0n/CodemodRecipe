//! component_graph `out` param + component_snapshot persistence under codemod root.

mod common;

use codemod_recipe_host::component::component_scan_inner;
use codemod_recipe_host::component::{persist_scan_snapshot, DEFAULT_SNAPSHOT_REL};
use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use common::{copy_dir_all, temp_workspace};
use std::path::PathBuf;

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../test/fixtures/reference_stack")
}

/// Isolate snapshot writes so parallel tests do not share `.cache/`.
fn isolated_registry() -> (PathBuf, RecipeRegistry) {
    let root = fixture_root();
    let tmp = temp_workspace("component_snapshot");
    let workspace = tmp.join("workspace");
    let codemod = tmp.join(".codemod");
    copy_dir_all(&root.join("workspace"), &workspace);
    copy_dir_all(&root.join(".codemod"), &codemod);
    let _ = std::fs::remove_dir_all(codemod.join(".cache"));
    let mut registry = RecipeRegistry::new(workspace, codemod);
    registry.reload();
    (tmp, registry)
}

#[test]
fn component_graph_out_writes_mermaid_and_rejects_traversal() {
    let root = fixture_root();
    let workspace = root.join("workspace");
    let codemod = root.join(".codemod");
    let mut registry = RecipeRegistry::new(workspace.clone(), codemod);
    registry.reload();

    let out_rel = "docs/_graph_test.mmd";
    let out_abs = workspace.join(out_rel);
    let _ = std::fs::remove_file(&out_abs);

    let resp = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentGraph {
            level: Some("kinds".to_string()),
            out: Some(out_rel.to_string()),
        },
    );
    assert_eq!(resp["ok"], true, "{resp}");
    assert_eq!(resp["path"], out_rel);
    let written = std::fs::read_to_string(&out_abs).expect("mermaid file");
    assert_eq!(written, resp["mermaid"].as_str().unwrap());
    let _ = std::fs::remove_file(&out_abs);

    let bad = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentGraph {
            level: None,
            out: Some("../outside.mmd".to_string()),
        },
    );
    assert_eq!(bad["ok"], false, "traversal must fail: {bad}");
}

#[test]
fn component_snapshot_defaults_to_codemod_cache_and_is_content_stable() {
    let (tmp, mut registry) = isolated_registry();
    let abs = tmp.join(".codemod").join(DEFAULT_SNAPSHOT_REL);

    let first =
        dispatch::handle_command(&mut registry, HostCommand::ComponentSnapshot { path: None });
    assert_eq!(first["ok"], true, "{first}");
    assert_eq!(first["path"], DEFAULT_SNAPSHOT_REL);
    assert_eq!(first["wrote"], true, "{first}");
    assert!(abs.exists(), "default snapshot missing");
    let kinds = tmp.join(".codemod").join(".cache/kind-graph.mmd");
    let instances = tmp.join(".codemod").join(".cache/instance-graph.mmd");
    assert!(kinds.exists(), "kind-graph.mmd missing");
    assert!(instances.exists(), "instance-graph.mmd missing");
    assert!(
        std::fs::read_to_string(&kinds)
            .unwrap()
            .starts_with("graph "),
        "kind mermaid"
    );
    let body1 = std::fs::read_to_string(&abs).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&body1).unwrap();
    assert!(parsed.get("meta").is_some());
    assert!(parsed.get("instances").is_some());
    // Top-level keys in file order: meta, then coverage (if any), relations, then the rest.
    // (Parsed Value objects are BTreeMaps — check the raw JSON prefix, not .keys().)
    let trimmed = body1.trim_start();
    assert!(
        trimmed.starts_with("{\n  \"meta\":") || trimmed.starts_with("{\n  \"meta\" :"),
        "snapshot must start with meta, got prefix: {}",
        &trimmed[..trimmed.len().min(80)]
    );
    let meta_at = trimmed.find("\"meta\"").expect("meta key");
    let relations_at = trimmed.find("\"relations\"").expect("relations key");
    assert!(
        meta_at < relations_at,
        "meta must appear before relations in file"
    );
    if let Some(cov_at) = trimmed.find("\"coverage\"") {
        assert!(meta_at < cov_at && cov_at < relations_at);
    }
    // reference_stack has no codebase → coverage omitted
    assert!(parsed.get("coverage").is_none());
    let relations = parsed
        .get("relations")
        .expect("relations metrics block");
    assert!(relations.get("aggregate").is_some(), "{relations}");
    assert!(relations["aggregate"].get("instanceCount").is_some());
    assert!(relations["aggregate"].get("relationIgnored").is_some());
    assert!(relations.get("byComponent").is_some());
    assert!(relations.get("byKind").is_some());
    let ts1 = parsed["meta"]["timestamp"].clone();

    let scan = component_scan_inner(&mut registry);
    let second = persist_scan_snapshot(&registry, &scan, None).expect("persist");
    assert!(!second.wrote, "noop must not rewrite");
    let body2 = std::fs::read_to_string(&abs).unwrap();
    assert_eq!(body1, body2);
    let parsed2: serde_json::Value = serde_json::from_str(&body2).unwrap();
    assert_eq!(parsed2["meta"]["timestamp"], ts1);

    let bad = dispatch::handle_command(
        &mut registry,
        HostCommand::ComponentSnapshot {
            path: Some("../escape.json".to_string()),
        },
    );
    assert_eq!(bad["ok"], false, "traversal must fail: {bad}");
}

#[test]
fn component_snapshot_includes_compact_coverage_when_codebase_set() {
    let (tmp, mut registry) = isolated_registry();
    let stack_path = tmp.join(".codemod/stack.luau");
    std::fs::write(
        &stack_path,
        r#"return {
  id = "reference-stack",
  components = { "example.entity", "example.store" },
  codebase = { "src/**/*.dart" },
}
"#,
    )
    .unwrap();
    registry.reload();

    let abs = tmp.join(".codemod").join(DEFAULT_SNAPSHOT_REL);
    let resp =
        dispatch::handle_command(&mut registry, HostCommand::ComponentSnapshot { path: None });
    assert_eq!(resp["ok"], true, "{resp}");
    let parsed: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&abs).unwrap()).unwrap();
    let cov = parsed
        .get("coverage")
        .expect("coverage key when codebase set");
    assert!(cov.get("aggregate").is_some(), "{cov}");
    assert!(cov["aggregate"].get("ratio").is_some());
    assert!(cov.get("files").and_then(|f| f.as_array()).is_some());
    // Compact: no gaps arrays on file entries
    for file in cov["files"].as_array().unwrap() {
        assert!(file.get("gaps").is_none(), "gaps must be omitted: {file}");
    }

    // Content-stable with coverage present
    let body1 = std::fs::read_to_string(&abs).unwrap();
    let scan = component_scan_inner(&mut registry);
    let second = persist_scan_snapshot(&registry, &scan, None).expect("persist");
    assert!(!second.wrote, "noop must not rewrite when coverage unchanged");
    assert_eq!(body1, std::fs::read_to_string(&abs).unwrap());
}

#[test]
fn component_scan_writes_default_snapshot() {
    let (tmp, mut registry) = isolated_registry();
    let abs = tmp.join(".codemod").join(DEFAULT_SNAPSHOT_REL);

    let scan = dispatch::handle_command(&mut registry, HostCommand::ComponentScan);
    assert_eq!(scan["ok"], true, "{scan}");
    assert!(abs.exists(), "scan should persist default snapshot");
}
