//! Query chain + query library resolution.

mod common;

use codemod_recipe_host::engine::runner::{collect_recipe_changes, parse_inline_recipe};
use codemod_recipe_host::lua_runtime::RunKind;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use common::{repo_root, temp_workspace};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Temp workspace seeded with the dart_demo query library and a `lib/foo.dart`
/// target file. Returns the workspace path and the reloaded registry.
fn workspace_with_dart_queries(name: &str) -> (PathBuf, RecipeRegistry) {
    let workspace = temp_workspace(name);
    let queries_dir = workspace.join(".codemod/queries");
    std::fs::create_dir_all(&queries_dir).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/dart_demo/.codemod/queries/dart.luau"),
        queries_dir.join("dart.luau"),
    )
    .unwrap();
    std::fs::create_dir_all(workspace.join("lib")).unwrap();
    std::fs::write(
        workspace.join("lib/foo.dart"),
        "class Settings {\n  void update() {}\n}\n",
    )
    .unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();
    (workspace, registry)
}

#[test]
fn chained_query_inserts_into_method_body() {
    let (_workspace, registry) = workspace_with_dart_queries("chain_insert");

    assert!(registry
        .queries_by_id()
        .contains_key("dart_queries.class_named"));

    let inline = r#"
return {
  id = "chain_test",
  args = {
    { name = "className", required = true },
    { name = "methodName", required = true },
  },
  apply = function(c, args)
    local classQuery = c:template({ inline = c:queryLib("dart_queries.class_named") })
    local methodQuery = c:template({ inline = c:queryLib("dart_queries.method_body") })
    c:insert({
      path = "lib/foo.dart",
      language = "dart",
      query = { classQuery, methodQuery },
      capture = "body",
      anchor = "end",
      text = "    print('codemod');\n",
    })
  end,
}
"#;

    let recipe = parse_inline_recipe(&serde_json::Value::String(inline.to_string())).unwrap();
    let args = BTreeMap::from([
        ("className".to_string(), serde_json::json!("Settings")),
        ("methodName".to_string(), serde_json::json!("update")),
    ]);

    let collected =
        collect_recipe_changes(&registry, &recipe, None, &args, RunKind::Preview).unwrap();
    let modified = collected
        .changes
        .iter()
        .find_map(|c| c.modified_content().ok().flatten())
        .expect("patch");

    assert!(modified.contains("print('codemod')"));
}

#[test]
fn chained_query_mixed_library_ref_and_inline_step() {
    let (_workspace, registry) = workspace_with_dart_queries("chain_mixed");

    let inline = r#"
return {
  id = "chain_mixed",
  args = {
    { name = "className", required = true },
    { name = "methodName", required = true },
  },
  apply = function(c, args)
    local classQuery = c:template({ inline = c:queryLib("dart_queries.class_named") })
    c:insert({
      path = "lib/foo.dart",
      language = "dart",
      query = { classQuery, "(function_body (block) @body)" },
      capture = "body",
      anchor = "end",
      text = "    print('mixed');\n",
    })
  end,
}
"#;

    let recipe = parse_inline_recipe(&serde_json::Value::String(inline.to_string())).unwrap();
    let args = BTreeMap::from([
        ("className".to_string(), serde_json::json!("Settings")),
        ("methodName".to_string(), serde_json::json!("update")),
    ]);

    let collected =
        collect_recipe_changes(&registry, &recipe, None, &args, RunKind::Preview).unwrap();
    let modified = collected
        .changes
        .iter()
        .find_map(|c| c.modified_content().ok().flatten())
        .expect("patch");

    assert!(modified.contains("print('mixed')"));
}
