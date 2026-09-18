use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use std::fs;
use std::path::PathBuf;

fn flutter_poc() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../test/fixtures/flutter_poc")
}

fn write_stack(codemod: &std::path::Path, codebase: Option<&str>) {
    let body = match codebase {
        Some(globs) => format!(
            r#"return {{
  id = "cov-stack",
  components = {{ "cov.root" }},
  codebase = {{ {globs} }},
}}
"#
        ),
        None => r#"return {
  id = "cov-stack",
  components = { "cov.root" },
}
"#
        .to_string(),
    };
    fs::write(codemod.join("stack.luau"), body).unwrap();
}

fn setup_coverage_workspace(name: &str, with_codebase: bool) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "codemod_cov_{}_{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    let workspace = root.join("workspace");
    let codemod = root.join(".codemod");
    let components = codemod.join("components");
    fs::create_dir_all(workspace.join("lib")).unwrap();
    fs::create_dir_all(&components).unwrap();

    // File: "ab\ncd" — cover "ab" (0..2), leave "cd" as gap (nonblank).
    fs::write(workspace.join("lib/a.dart"), "ab\ncd").unwrap();
    fs::write(workspace.join("lib/b.dart"), "xy").unwrap();

    write_stack(
        &codemod,
        if with_codebase {
            Some(r#""lib/*.dart""#)
        } else {
            None
        },
    );

    fs::write(
        components.join("cov.root.luau"),
        r#"
return {
  id = "cov.root",
  identify = function(c)
    return { { path = "lib/a.dart" } }
  end,
  parse = function(c, locator)
    return {
      id = "a",
      source = { path = locator.path, span = { from = 0, to = 2 } },
      children = {},
    }
  end,
  verify = function(_c, _i)
    return {}
  end,
}
"#,
    )
    .unwrap();

    root
}

#[test]
fn coverage_skipped_without_codebase() {
    let fixture = flutter_poc();
    let mut registry = RecipeRegistry::new(
        fixture.join("workspace"),
        fixture.join(".codemod"),
    );
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::ComponentCoverage);
    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(response["skipped"], true);
    let warning = response["warning"].as_str().unwrap_or("");
    assert!(
        warning.contains("codebase"),
        "expected codebase warning, got {warning}"
    );
    assert!(response.get("files").is_none() || response["files"].is_null());
}

#[test]
fn coverage_reports_per_file_gaps_and_aggregate() {
    let root = setup_coverage_workspace("gaps", true);
    let mut registry = RecipeRegistry::new(root.join("workspace"), root.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::ComponentCoverage);
    assert_eq!(response["ok"], true, "{response}");
    assert!(response.get("skipped").is_none() || response["skipped"].is_null());

    let files = response["files"].as_array().expect("files");
    assert_eq!(files.len(), 2);

    let a = files
        .iter()
        .find(|f| f["path"] == "lib/a.dart")
        .expect("lib/a.dart");
    assert_eq!(a["coveredNonblank"], 2);
    assert_eq!(a["gapNonblank"], 2); // "cd"
    assert!((a["ratio"].as_f64().unwrap() - 0.5).abs() < 1e-9);
    let gaps = a["gaps"].as_array().unwrap();
    assert_eq!(gaps.len(), 1);
    assert_eq!(gaps[0]["from"], 3);
    assert_eq!(gaps[0]["to"], 5);

    let b = files
        .iter()
        .find(|f| f["path"] == "lib/b.dart")
        .expect("lib/b.dart");
    assert_eq!(b["coveredNonblank"], 0);
    assert_eq!(b["gapNonblank"], 2); // "xy" entirely uncovered
    assert_eq!(b["ratio"], 0.0);

    let agg = &response["aggregate"];
    assert_eq!(agg["coveredNonblank"], 2);
    assert_eq!(agg["gapNonblank"], 4);
    assert!((agg["ratio"].as_f64().unwrap() - (2.0 / 6.0)).abs() < 1e-9);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn coverage_skipped_when_codebase_empty_on_minimal_stack() {
    let root = setup_coverage_workspace("ncb", false);
    let mut registry = RecipeRegistry::new(root.join("workspace"), root.join(".codemod"));
    registry.reload();

    let response = dispatch::handle_command(&mut registry, HostCommand::ComponentCoverage);
    assert_eq!(response["ok"], true, "{response}");
    assert_eq!(response["skipped"], true);
    assert!(response["warning"]
        .as_str()
        .unwrap_or("")
        .contains("codebase"));

    let _ = fs::remove_dir_all(root);
}
