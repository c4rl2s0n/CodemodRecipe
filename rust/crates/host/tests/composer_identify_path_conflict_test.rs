//! `composer.identify_path_conflict` when two stack roots identify the same path.

mod common;

use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use std::fs;

#[test]
fn dual_root_identify_same_path_emits_conflict() {
    let root = common::temp_workspace("identify_path_conflict");
    let _ = fs::remove_dir_all(&root);
    let workspace = root.join("workspace");
    let codemod = root.join(".codemod");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(codemod.join("components")).unwrap();
    fs::write(workspace.join("src").join("shared.rs"), "fn x() {}\n").unwrap();

    fs::write(
        codemod.join("stack.luau"),
        r#"
return {
  id = "test-stack",
  components = { "example.a", "example.b" },
}
"#,
    )
    .unwrap();

    let component = |id: &str| {
        format!(
            r#"
return {{
  id = "{id}",
  description = "dual-root conflict fixture",
  identify = function(_c)
    return {{ {{ path = "src/shared.rs" }} }}
  end,
  parse = function(_c, locator)
    return {{ id = "node", componentId = "{id}", source = locator }}
  end,
}}
"#
        )
    };
    fs::write(
        codemod.join("components").join("example.a.luau"),
        component("example.a"),
    )
    .unwrap();
    fs::write(
        codemod.join("components").join("example.b.luau"),
        component("example.b"),
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace, codemod);
    registry.reload();
    let response = dispatch::handle_command(&mut registry, HostCommand::ComponentScan);
    assert_eq!(response["ok"], true, "{response}");
    let findings = response["findings"].as_array().expect("findings");
    let codes: Vec<&str> = findings
        .iter()
        .filter_map(|f| f["code"].as_str())
        .collect();
    assert!(
        codes.contains(&"composer.identify_path_conflict"),
        "expected composer.identify_path_conflict, got {codes:?}"
    );
}

#[test]
fn rust_test_module_may_share_path_with_any_owner() {
    let root = common::temp_workspace("identify_path_share_tests");
    let _ = fs::remove_dir_all(&root);
    let workspace = root.join("workspace");
    let codemod = root.join(".codemod");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(codemod.join("components")).unwrap();
    fs::write(workspace.join("src").join("lib.rs"), "fn x() {}\n").unwrap();

    fs::write(
        codemod.join("stack.luau"),
        r#"
return {
  id = "test-stack",
  components = { "repo.component_pipeline", "repo.rust_test_module" },
}
"#,
    )
    .unwrap();

    let component = |id: &str| {
        format!(
            r#"
return {{
  id = "{id}",
  description = "path-share allowlist fixture",
  identify = function(_c)
    return {{ {{ path = "src/lib.rs" }} }}
  end,
  parse = function(_c, locator)
    return {{ id = "node", componentId = "{id}", source = locator }}
  end,
}}
"#
        )
    };
    fs::write(
        codemod.join("components").join("repo.component_pipeline.luau"),
        component("repo.component_pipeline"),
    )
    .unwrap();
    fs::write(
        codemod.join("components").join("repo.rust_test_module.luau"),
        component("repo.rust_test_module"),
    )
    .unwrap();

    let mut registry = RecipeRegistry::new(workspace, codemod);
    registry.reload();
    let response = dispatch::handle_command(&mut registry, HostCommand::ComponentScan);
    assert_eq!(response["ok"], true, "{response}");
    let findings = response["findings"].as_array().expect("findings");
    let codes: Vec<&str> = findings
        .iter()
        .filter_map(|f| f["code"].as_str())
        .collect();
    assert!(
        !codes.contains(&"composer.identify_path_conflict"),
        "domain↔test_module path share must not conflict, got {codes:?}"
    );
}
