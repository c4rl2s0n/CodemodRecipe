//! Stack-level operations: fan-out atomicity, cycle guard, unknown-op error.

use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

fn temp_workspace(name: &str) -> PathBuf {
    let n = N.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!(
        "codemod_stack_ops_{name}_{}_{n}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join(".codemod/components")).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    root
}

fn write(root: &PathBuf, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, content).unwrap();
}

fn registry_for(root: &PathBuf) -> RecipeRegistry {
    let mut registry = RecipeRegistry::new(root.clone(), root.join(".codemod"));
    registry.reload();
    registry
}

#[test]
fn stack_operation_fan_out_is_atomic_across_layers() {
    let root = temp_workspace("fanout");
    write(
        &root,
        ".codemod/components/layer_a.luau",
        r#"
return {
  id = "layer_a",
  identify = function(c)
    local out = {}
    for _, path in ipairs(c:listFiles("src/a.txt")) do
      out[#out + 1] = { path = path }
    end
    return out
  end,
  parse = function(_c, locator)
    return { id = "a", componentId = "layer_a", source = { path = locator.path } }
  end,
  operations = {
    createCommand = {
      args = { { name = "name", required = true } },
      apply = function(c, args, _target, _graph)
        c:create({ path = "src/out_a.txt", content = "A:" .. tostring(args.name) .. "\n" })
      end,
    },
  },
}
"#,
    );
    write(
        &root,
        ".codemod/components/layer_b.luau",
        r#"
return {
  id = "layer_b",
  identify = function(c)
    local out = {}
    for _, path in ipairs(c:listFiles("src/b.txt")) do
      out[#out + 1] = { path = path }
    end
    return out
  end,
  parse = function(_c, locator)
    return { id = "b", componentId = "layer_b", source = { path = locator.path } }
  end,
  operations = {
    createCommand = {
      args = { { name = "name", required = true } },
      apply = function(c, args, _target, _graph)
        c:create({ path = "src/out_b.txt", content = "B:" .. tostring(args.name) .. "\n" })
      end,
    },
  },
}
"#,
    );
    write(
        &root,
        ".codemod/stack_ops.luau",
        r#"
return {
  operations = {
    createCommand = {
      args = {
        { name = "name", required = true },
      },
      apply = function(c, args, _graph)
        c:component("layer_a"):operation("createCommand", args)
        c:component("layer_b"):operation("createCommand", args)
      end,
    },
  },
}
"#,
    );
    write(
        &root,
        ".codemod/stack.luau",
        r#"
local ops = require("./stack_ops")
return {
  id = "test-stack",
  components = { "layer_a", "layer_b" },
  operations = ops.operations,
}
"#,
    );
    write(&root, "src/a.txt", "base-a\n");
    write(&root, "src/b.txt", "base-b\n");

    let mut registry = registry_for(&root);

    // Preview: one diff across both layers, no disk writes
    let preview = dispatch::handle_command(
        &mut registry,
        HostCommand::StackInvokeOperation {
            operation: "createCommand".to_string(),
            preview: true,
            args: [("name".to_string(), serde_json::json!("Snap"))]
                .into_iter()
                .collect(),
                    preview_token: None,
        },
    );
    assert_eq!(preview["ok"], true, "preview failed: {preview}");
    let files = preview["files"].as_array().expect("files");
    assert_eq!(
        files.len(),
        2,
        "expected both layers in one preview: {preview}"
    );
    assert!(
        !root.join("src/out_a.txt").exists(),
        "preview must not write"
    );
    assert!(
        !root.join("src/out_b.txt").exists(),
        "preview must not write"
    );

    // Apply writes both (requires previewToken from dry-run)
    let token = preview["previewToken"]
        .as_str()
        .expect("previewToken")
        .to_string();
    let apply = dispatch::handle_command(
        &mut registry,
        HostCommand::StackInvokeOperation {
            operation: "createCommand".to_string(),
            preview: false,
            args: [("name".to_string(), serde_json::json!("Snap"))]
                .into_iter()
                .collect(),
            preview_token: Some(token),
        },
    );
    assert_eq!(apply["ok"], true, "apply failed: {apply}");
    assert_eq!(
        std::fs::read_to_string(root.join("src/out_a.txt")).unwrap(),
        "A:Snap\n"
    );
    assert_eq!(
        std::fs::read_to_string(root.join("src/out_b.txt")).unwrap(),
        "B:Snap\n"
    );

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn stack_unknown_operation_errors() {
    let root = temp_workspace("unknown");
    write(
        &root,
        ".codemod/components/layer_a.luau",
        r#"
return {
  id = "layer_a",
  identify = function() return {} end,
  parse = function() return { id = "a" } end,
}
"#,
    );
    write(
        &root,
        ".codemod/stack.luau",
        r#"
return {
  id = "test-stack",
  components = { "layer_a" },
  operations = {
    createCommand = {
      apply = function() end,
    },
  },
}
"#,
    );
    let mut registry = registry_for(&root);
    let resp = dispatch::handle_command(
        &mut registry,
        HostCommand::StackInvokeOperation {
            operation: "noSuchOp".to_string(),
            preview: true,
            args: Default::default(),
                    preview_token: None,
        },
    );
    assert_eq!(resp["ok"], false);
    let err = resp["error"].as_str().unwrap_or("");
    assert!(
        err.contains("unknown stack operation"),
        "unexpected error: {err}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn stack_operation_cycle_guard() {
    let root = temp_workspace("cycle");
    write(
        &root,
        ".codemod/components/layer_a.luau",
        r#"
return {
  id = "layer_a",
  identify = function(c)
    local out = {}
    for _, path in ipairs(c:listFiles("src/a.txt")) do
      out[#out + 1] = { path = path }
    end
    return out
  end,
  parse = function(_c, locator)
    return { id = "a", componentId = "layer_a", source = { path = locator.path } }
  end,
  operations = {
    createCommand = {
      apply = function(c, args, _target, _graph)
        -- re-enter self via delegate → cycle
        c:component("layer_a"):operation("createCommand", args)
      end,
    },
  },
}
"#,
    );
    write(
        &root,
        ".codemod/stack.luau",
        r#"
return {
  id = "test-stack",
  components = { "layer_a" },
  operations = {
    createCommand = {
      apply = function(c, args, _graph)
        c:component("layer_a"):operation("createCommand", args)
      end,
    },
  },
}
"#,
    );
    write(&root, "src/a.txt", "base\n");
    let mut registry = registry_for(&root);
    let resp = dispatch::handle_command(
        &mut registry,
        HostCommand::StackInvokeOperation {
            operation: "createCommand".to_string(),
            preview: true,
            args: Default::default(),
                    preview_token: None,
        },
    );
    assert_eq!(resp["ok"], false, "cycle should fail: {resp}");
    let err = resp["error"].as_str().unwrap_or("");
    assert!(err.contains("cycle"), "expected cycle error, got: {err}");
    let _ = std::fs::remove_dir_all(&root);
}
