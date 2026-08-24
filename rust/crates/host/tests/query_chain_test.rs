//! Query chain + query library resolution.

use codemod_recipe_host::engine::runner::{collect_recipe_changes, parse_inline_recipe};
use codemod_recipe_host::lua_runtime::RunKind;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn chained_query_inserts_into_method_body() {
    let workspace = repo_root();
    let codemod = workspace.join(".codemod");
    let mut registry = RecipeRegistry::new(workspace.clone(), codemod);
    registry.reload();

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
  apply = function(c)
    local classQuery = c:applyTemplate({ template = c:queryLib("dart_queries.class_named") })
    local methodQuery = c:applyTemplate({ template = c:queryLib("dart_queries.method_body") })
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

    let source = "class Settings {\n  void update() {}\n}\n";
    std::fs::create_dir_all(workspace.join("lib")).ok();
    std::fs::write(workspace.join("lib/foo.dart"), source).unwrap();

    let collected =
        collect_recipe_changes(&registry, &recipe, None, &args, RunKind::Preview).unwrap();
    let modified = collected
        .changes
        .iter()
        .find_map(|c| c.modified_content().ok().flatten())
        .expect("patch");

    assert!(modified.contains("print('codemod')"));

    let _ = std::fs::remove_file(workspace.join("lib/foo.dart"));
}

#[test]
fn chained_query_mixed_library_ref_and_inline_step() {
    let workspace = repo_root();
    let codemod = workspace.join(".codemod");
    let mut registry = RecipeRegistry::new(workspace.clone(), codemod);
    registry.reload();

    let inline = r#"
return {
  id = "chain_mixed",
  args = {
    { name = "className", required = true },
    { name = "methodName", required = true },
  },
  apply = function(c)
    local classQuery = c:applyTemplate({ template = c:queryLib("dart_queries.class_named") })
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

    let source = "class Settings {\n  void update() {}\n}\n";
    std::fs::create_dir_all(workspace.join("lib")).ok();
    std::fs::write(workspace.join("lib/foo.dart"), source).unwrap();

    let collected =
        collect_recipe_changes(&registry, &recipe, None, &args, RunKind::Preview).unwrap();
    let modified = collected
        .changes
        .iter()
        .find_map(|c| c.modified_content().ok().flatten())
        .expect("patch");

    assert!(modified.contains("print('mixed')"));

    let _ = std::fs::remove_file(workspace.join("lib/foo.dart"));
}
