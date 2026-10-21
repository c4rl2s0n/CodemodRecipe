//! `composer.identify_empty` when a stack root identify returns no locators.

mod common;

use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use std::fs;

fn write_stack_and_component(
    name: &str,
    component_src: &str,
    stack_component_id: &str,
) -> (std::path::PathBuf, std::path::PathBuf) {
    let root = common::temp_workspace(name);
    let _ = fs::remove_dir_all(&root);
    let workspace = root.join("workspace");
    let codemod = root.join(".codemod");
    fs::create_dir_all(workspace.join("src")).unwrap();
    fs::create_dir_all(codemod.join("components")).unwrap();

    fs::write(
        codemod.join("stack.luau"),
        format!(
            r#"
return {{
  id = "test-stack",
  components = {{ "{stack_component_id}" }},
}}
"#
        ),
    )
    .unwrap();
    fs::write(
        codemod.join("components").join(format!("{stack_component_id}.luau")),
        component_src,
    )
    .unwrap();
    (workspace, codemod)
}

#[test]
fn empty_identify_emits_composer_identify_empty() {
    let (workspace, codemod) = write_stack_and_component(
        "identify_empty",
        r#"
return {
  id = "example.empty",
  description = "intentionally empty identify for guard test",
  identify = function(_c)
    return {}
  end,
  parse = function(_c, locator)
    return { id = "x", componentId = "example.empty", source = locator }
  end,
}
"#,
        "example.empty",
    );

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
        codes.contains(&"composer.identify_empty"),
        "expected composer.identify_empty, got {codes:?}"
    );
    let summary = response.get("summary").expect("summary");
    assert!(
        summary["deadComposers"]
            .as_array()
            .map(|a| a.iter().any(|v| v.as_str() == Some("example.empty")))
            .unwrap_or(false),
        "summary.deadComposers should list example.empty: {summary}"
    );
}
