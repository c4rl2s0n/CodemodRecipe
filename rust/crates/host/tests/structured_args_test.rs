//! JSON list/object recipe args round-trip through preview.

use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn temp_workspace(name: &str) -> PathBuf {
    let n = N.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("structured_args_{name}_{}_{n}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("lib")).unwrap();
    std::fs::create_dir_all(dir.join(".codemod/recipes")).unwrap();
    dir
}

#[test]
fn preview_list_of_structs_in_apply() {
    let workspace = temp_workspace("list_structs");
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = r#"
return {
  id = "write_properties",
  types = {
    ClassProperty = {
      fields = {
        { name = "type", inputKind = "text", required = true },
        { name = "name", inputKind = "text", required = true },
      },
    },
  },
  args = {
    { name = "properties", inputKind = { "ClassProperty" } },
  },
  apply = function(c)
    local lines = {}
    for _, p in ipairs(c.args.properties or {}) do
      table.insert(lines, p.type .. " " .. p.name)
    end
    c:create({
      path = "lib/generated.dart",
      content = table.concat(lines, "\n") .. "\n",
    })
  end,
}
"#;

    let mut args = BTreeMap::new();
    args.insert(
        "properties".to_string(),
        serde_json::json!([
            { "type": "int", "name": "count" },
            { "type": "String", "name": "label" }
        ]),
    );

    let response = dispatch::handle_command(
        &mut registry,
        HostCommand::Preview {
            recipe: None,
            inline_recipe: Some(serde_json::Value::String(inline.to_string())),
            args,
            snippet_lines: Some(20),
        },
    );
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let modified = response["files"][0]["modified"].as_str().unwrap();
    assert!(modified.contains("int count"));
    assert!(modified.contains("String label"));
}
