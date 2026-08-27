//! c:log and partial transcript on apply failure.

mod common;

use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use common::temp_workspace;
use std::collections::BTreeMap;

fn preview_inline(
    registry: &mut RecipeRegistry,
    source: serde_json::Value,
    args: BTreeMap<String, serde_json::Value>,
) -> serde_json::Value {
    dispatch::handle_command(
        registry,
        HostCommand::Preview {
            recipe: None,
            inline_recipe: Some(source),
            args,
            snippet_lines: None,
        },
    )
}

#[test]
fn log_events_and_partial_transcript_on_failure() {
    let workspace = temp_workspace("log_fail_transcript");
    std::fs::create_dir_all(workspace.join("lib")).unwrap();
    std::fs::create_dir_all(workspace.join(".codemod/recipes")).unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!(
        r#"
return {
  id = "log_then_fail",
  apply = function(c)
    c:log("before", { n = 1, path = "x" })
    c:insert({
      path = "missing.dart",
      query = "(program) @root",
      capture = "root",
      anchor = "end",
      text = "x",
    })
  end,
}
"#
    );

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], false, "{}", response);
    let transcript = response["transcript"].as_array().expect("transcript");
    assert!(
        transcript.iter().any(|e| e["kind"] == "log"),
        "expected log event: {transcript:?}"
    );
    let log = transcript.iter().find(|e| e["kind"] == "log").unwrap();
    let detail = log["detail"].as_str().unwrap_or("");
    assert!(detail.contains("before"), "detail={detail}");
    assert!(
        detail.contains("\"n\":1") || detail.contains("n"),
        "detail={detail}"
    );
    assert_eq!(log["expandable"], true, "{log}");
    assert!(log["payload"]["json"].is_object() || log["payload"].is_object(), "{log}");

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn insert_transcript_includes_stats_and_payload() {
    let workspace = temp_workspace("insert_transcript_stats");
    std::fs::create_dir_all(workspace.join("lib")).unwrap();
    std::fs::write(workspace.join("lib/a.dart"), "class A {}\n").unwrap();
    std::fs::create_dir_all(workspace.join(".codemod/recipes")).unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!(
        r#"
return {
  id = "insert_stats",
  apply = function(c)
    c:insert({
      path = "lib/a.dart",
      query = "(class_definition) @cls",
      capture = "cls",
      anchor = "end",
      text = "\n// one\n// two\n",
    })
  end,
}
"#
    );

    let response = preview_inline(&mut registry, inline, BTreeMap::new());
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let transcript = response["transcript"].as_array().expect("transcript");
    let insert = transcript
        .iter()
        .find(|e| e["kind"] == "insert")
        .expect("insert");
    assert_eq!(insert["detail"], "lib/a.dart");
    assert!(
        insert["stats"]["addedLines"].as_i64().unwrap_or(0) >= 2,
        "{insert}"
    );
    assert_eq!(insert["payload"]["capture"], "cls");
    assert_eq!(insert["payload"]["anchor"], "end");
    assert_eq!(insert["expandable"], true);

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn template_inline_and_path() {
    let workspace = temp_workspace("template_api");
    std::fs::create_dir_all(workspace.join("lib")).unwrap();
    std::fs::create_dir_all(workspace.join(".codemod/templates")).unwrap();
    std::fs::create_dir_all(workspace.join(".codemod/recipes")).unwrap();
    std::fs::write(
        workspace.join(".codemod/templates/hi.dart.template"),
        "class {{ name | pascal_case }} {}\n",
    )
    .unwrap();
    let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
    registry.reload();

    let inline = serde_json::json!(
        r#"
return {
  id = "template_api",
  args = { { name = "name", required = true } },
  apply = function(c)
    c:create({
      path = "lib/a.dart",
      content = c:template({ path = "templates/hi.dart.template" }),
      ifExists = "fail",
    })
    c:create({
      path = "lib/b.dart",
      content = c:template({ inline = "class {{ name | pascal_case }}B {}\n" }),
      ifExists = "fail",
    })
  end,
}
"#
    );
    let mut args = BTreeMap::new();
    args.insert("name".into(), serde_json::json!("widget"));
    let response = preview_inline(&mut registry, inline, args);
    assert_eq!(response["ok"], true, "{}", response["error"]);
    let files = response["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);

    let transcript = response["transcript"].as_array().expect("transcript");
    let template_events: Vec<_> = transcript
        .iter()
        .filter(|e| e["kind"] == "template")
        .collect();
    assert_eq!(template_events.len(), 2, "{transcript:?}");
    assert!(
        template_events
            .iter()
            .any(|e| e["detail"].as_str().unwrap_or("").starts_with("path=")),
        "{template_events:?}"
    );
    assert!(
        template_events.iter().any(|e| e["detail"] == "inline"),
        "{template_events:?}"
    );
    assert!(
        template_events.iter().all(|e| e["expandable"] == true),
        "{template_events:?}"
    );
    let create = transcript
        .iter()
        .find(|e| e["kind"] == "create")
        .expect("create event");
    assert!(
        create["stats"]["addedLines"].as_i64().unwrap_or(0) > 0,
        "{create}"
    );

    let _ = std::fs::remove_dir_all(workspace);
}
