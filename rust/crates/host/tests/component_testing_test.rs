//! Testing layer: `*.test.luau` suites — load/classify, registration, pass/fail
//! reporting with line numbers, fixture isolation (real workspace untouched,
//! temp dirs cleaned), base+override fixtures, scan/apply/recipe flows, filters.

mod common;

use codemod_recipe_host::protocol::dispatch;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::recipe::registry::RecipeRegistry;
use common::temp_workspace;
use std::fs;
use std::path::{Path, PathBuf};

const STACK: &str = r#"
return {
	id = "t-stack",
	components = { "t.model", "t.store" },
}
"#;

const MODEL: &str = r#"
return {
	id = "t.model",
	identify = function(c)
		local out = {}
		for _, row in ipairs(c:matchFiles("lib/models/{name}.dart")) do
			out[#out + 1] = { path = row.path, name = row.name }
		end
		return out
	end,
	parse = function(_c, locator)
		return { id = locator.name, path = locator.path }
	end,
	scaffold = {
		args = { { name = "name", required = true } },
		apply = function(c, args)
			c:create({
				path = "lib/models/" .. args.name .. ".dart",
				content = "class " .. args.name .. " {}\n",
			})
		end,
	},
	operations = {
		touch = {
			args = { { name = "name", required = true } },
			apply = function(c, args)
				c:create({ path = "out/" .. args.name .. ".txt", content = "x" })
			end,
		},
	},
}
"#;

const STORE: &str = r#"
return {
	id = "t.store",
	identify = function(c)
		local out = {}
		for _, row in ipairs(c:matchFiles("lib/stores/{id}_store.dart")) do
			out[#out + 1] = { path = row.path, id = row.id }
		end
		return out
	end,
	parse = function(_c, locator)
		return { id = locator.id or "unknown", path = locator.path }
	end,
	relate = function(_c, instance, _index)
		return {
			kind = "stores",
			to = { componentId = "t.model", key = instance.ast.id },
		}
	end,
}
"#;

const RECIPE: &str = r#"
return {
	id = "t.make_file",
	args = { { name = "name", required = true } },
	apply = function(c, args)
		c:create({ path = "gen/" .. args.name .. ".txt", content = "hello " .. args.name })
	end,
}
"#;

const MODEL_TESTS: &str = r#"
return {
	id = "t.model.tests",
	target = "t.model",
	tests = {
		parses_model = function(t)
			local ws = t:workspace({ files = { ["lib/models/recipe.dart"] = "class Recipe {}\n" } })
			local node = ws:parse("t.model", { path = "lib/models/recipe.dart", name = "recipe" })
			t:assertEq(node.id, "recipe")
			t:assertEq(node.path, "lib/models/recipe.dart")
		end,

		identify_finds_models = function(t)
			local ws = t:workspace({ files = {
				["lib/models/a.dart"] = "class A {}\n",
				["lib/models/b.dart"] = "class B {}\n",
			} })
			t:assertEq(#ws:identify("t.model"), 2)
		end,

		operation_mutates_fixture = function(t)
			local ws = t:workspace({ files = { ["lib/models/recipe.dart"] = "class Recipe {}\n" } })
			local scan = ws:scan()
			t:assertEq(#scan.instances, 1)
			ws:apply("t.model", "touch", { name = "x" }, "t.model/recipe")
			t:assertEq(ws:readFile("out/x.txt"), "x")
		end,

		recipe_applies_and_second_apply_fails = function(t)
			local ws = t:workspace({})
			local res = ws:applyRecipe("t.make_file", { name = "a" })
			t:assertEq(ws:readFile("gen/a.txt"), "hello a")
			t:assertEq(#res.files, 1)
			-- The fixture was flushed: a second apply hits the existing file.
			local ok, err = pcall(function()
				ws:applyRecipe("t.make_file", { name = "a" })
			end)
			t:assert(not ok, "second apply must fail")
			t:assert(tostring(err):find("already exists") ~= nil, tostring(err))
		end,

		base_dir_with_inline_override = function(t)
			local ws = t:workspace({
				base = "fixtures/with_model",
				files = { ["lib/stores/recipe_store.dart"] = "class RecipeStore {}\n" },
			})
			local scan = ws:scan()
			t:assertEq(#scan.instances, 2)
			t:assertEq(#scan.edges, 1)
		end,

		fails_with_line = function(t)
			t:assertEq(1, 2)
		end,

		plain_assert_fails = function(t)
			assert(false, "boom")
		end,
	},
}
"#;

const STACK_TESTS: &str = r#"
return {
	id = "t-stack.tests",
	tests = {
		store_relation_resolves = function(t)
			local ws = t:workspace({ files = {
				["lib/models/recipe.dart"] = "class Recipe {}\n",
				["lib/stores/recipe_store.dart"] = "class RecipeStore {}\n",
			} })
			local scan = ws:scan()
			t:assertEq(#scan.instances, 2)
			t:assertEq(#scan.edges, 1)
			t:assertEq(scan.edges[1].kind, "stores")
			t:assertEq(scan.edges[1].from, "t.store/recipe")
			t:assertEq(scan.edges[1].to, "t.model/recipe")
		end,

		scaffold_then_scan = function(t)
			local ws = t:workspace({})
			ws:scaffold("t.model", { name = "user" })
			t:assertEq(ws:readFile("lib/models/user.dart"), "class user {}\n")
			local scan = ws:scan()
			t:assertEq(#scan.instances, 1)
			t:assertEq(scan.instances[1].id, "t.model/user")
		end,
	},
}
"#;

fn setup_testing_workspace(name: &str) -> PathBuf {
    let root = temp_workspace(name);
    let _ = fs::remove_dir_all(&root); // clean slate (temp dirs are pid-scoped)
    let workspace = root.join("workspace");
    let codemod = root.join(".codemod");
    fs::create_dir_all(workspace.join("lib/models")).unwrap();
    fs::create_dir_all(codemod.join("components")).unwrap();
    fs::create_dir_all(codemod.join("recipes")).unwrap();
    // Fixture dir for base+override tests (never loaded as an asset).
    let fixture = codemod.join("tests/fixtures/with_model");
    fs::create_dir_all(fixture.join("lib/models")).unwrap();
    fs::write(fixture.join("lib/models/recipe.dart"), "class Recipe {}\n").unwrap();

    fs::write(codemod.join("stack.luau"), STACK).unwrap();
    fs::write(codemod.join("components/t.model.luau"), MODEL).unwrap();
    fs::write(codemod.join("components/t.store.luau"), STORE).unwrap();
    fs::write(codemod.join("recipes/t.make_file.luau"), RECIPE).unwrap();
    fs::write(codemod.join("components/t.model.test.luau"), MODEL_TESTS).unwrap();
    fs::write(codemod.join("stack.test.luau"), STACK_TESTS).unwrap();

    // The real workspace stays empty: suites must never touch it.
    fs::write(workspace.join("lib/models/real.dart"), "class Real {}\n").unwrap();
    root
}

fn registry_for(root: &Path) -> RecipeRegistry {
    let mut registry = RecipeRegistry::new(root.join("workspace"), root.join(".codemod"));
    registry.reload();
    registry
}

fn run(registry: &mut RecipeRegistry, filter: Option<&str>) -> serde_json::Value {
    dispatch::handle_command(
        registry,
        HostCommand::RunTests {
            filter: filter.map(str::to_string),
        },
    )
}

fn suite<'a>(report: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    report["suites"]
        .as_array()
        .expect("suites")
        .iter()
        .find(|s| s["id"] == id)
        .unwrap_or_else(|| panic!("suite {id} in {report}"))
}

fn test_case<'a>(suite: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    suite["tests"]
        .as_array()
        .expect("tests")
        .iter()
        .find(|t| t["name"] == name)
        .unwrap_or_else(|| panic!("test {name} in {suite}"))
}

#[test]
fn suites_register_in_catalog() {
    let root = setup_testing_workspace("testing_catalog");
    let registry = registry_for(&root);
    let suites = registry.test_suites();
    assert!(suites.contains_key("t.model.tests"));
    assert!(suites.contains_key("t-stack.tests"));
    assert_eq!(suites["t.model.tests"].target.as_deref(), Some("t.model"));
    assert_eq!(suites["t.model.tests"].test_names.len(), 7);
    assert_eq!(suites["t-stack.tests"].test_names.len(), 2);
}

#[test]
fn list_catalog_carries_test_suites() {
    let root = setup_testing_workspace("testing_catalog_list");
    let mut registry = registry_for(&root);
    let catalog = dispatch::handle_command(&mut registry, HostCommand::List);
    let suites = catalog["testSuites"].as_array().expect("testSuites");
    assert_eq!(suites.len(), 2, "{catalog}");

    let model = suites
        .iter()
        .find(|s| s["id"] == "t.model.tests")
        .expect("t.model.tests");
    assert_eq!(model["target"], "t.model");
    // The fixture keeps `.codemod` as a sibling of `workspace/`, so the path
    // does not strip to a relative one here; in consumer layouts (`.codemod`
    // inside the workspace root) it is workspace-relative.
    assert!(
        model["path"]
            .as_str()
            .expect("path")
            .ends_with(".codemod/components/t.model.test.luau"),
        "{catalog}"
    );
    // Sorted names, matching loader ordering.
    let names: Vec<&str> = model["testNames"]
        .as_array()
        .expect("testNames")
        .iter()
        .map(|n| n.as_str().unwrap())
        .collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted);
    assert_eq!(names.len(), 7);
}

#[test]
fn passing_tests_cover_parse_identify_scan_apply_recipe_scaffold() {
    let root = setup_testing_workspace("testing_pass");
    let mut registry = registry_for(&root);

    // Everything except the two intentionally-failing tests.
    let report = run(&mut registry, None);
    assert_eq!(report["ok"], false, "two tests fail on purpose: {report}");
    assert_eq!(report["totals"]["passed"], 7);
    assert_eq!(report["totals"]["failed"], 2);

    let model = suite(&report, "t.model.tests");
    for name in [
        "parses_model",
        "identify_finds_models",
        "operation_mutates_fixture",
        "recipe_applies_and_second_apply_fails",
        "base_dir_with_inline_override",
    ] {
        assert_eq!(
            test_case(model, name)["ok"],
            true,
            "{name} failed: {report}"
        );
    }
    let stack = suite(&report, "t-stack.tests");
    for name in ["store_relation_resolves", "scaffold_then_scan"] {
        assert_eq!(
            test_case(stack, name)["ok"],
            true,
            "{name} failed: {report}"
        );
    }
}

#[test]
fn failing_tests_report_error_and_line() {
    let root = setup_testing_workspace("testing_fail");
    let mut registry = registry_for(&root);

    let report = run(&mut registry, Some("t.model.tests/fails_with_line"));
    assert_eq!(report["ok"], false);
    assert_eq!(report["totals"]["failed"], 1);
    let case = &suite(&report, "t.model.tests")["tests"][0];
    let error = case["error"].as_str().expect("error message");
    assert!(
        error.contains("expected 2, got 1"),
        "assertEq renders values: {error}"
    );
    let expected_line = MODEL_TESTS
        .lines()
        .position(|l| l.contains("t:assertEq(1, 2)"))
        .unwrap()
        + 1;
    assert_eq!(
        case["line"].as_u64(),
        Some(expected_line as u64),
        "failure line points at the test file: {error}"
    );

    let report = run(&mut registry, Some("plain_assert_fails"));
    let case = &suite(&report, "t.model.tests")["tests"][0];
    let error = case["error"].as_str().expect("error message");
    assert!(error.contains("boom"), "plain assert message: {error}");
    assert!(
        case["line"].as_u64().is_some(),
        "plain assert line: {error}"
    );
}

#[test]
fn filter_selects_suites_and_tests() {
    let root = setup_testing_workspace("testing_filter");
    let mut registry = registry_for(&root);

    // By suite id: all 7 tests of t.model.tests.
    let report = run(&mut registry, Some("t.model.tests"));
    assert_eq!(report["suites"].as_array().unwrap().len(), 1);
    let passed = report["totals"]["passed"].as_u64().unwrap();
    let failed = report["totals"]["failed"].as_u64().unwrap();
    assert_eq!(passed + failed, 7);

    // By suite target.
    let report = run(&mut registry, Some("t.model"));
    assert_eq!(report["suites"].as_array().unwrap().len(), 1);
    assert_eq!(
        suite(&report, "t.model.tests")["tests"]
            .as_array()
            .unwrap()
            .len(),
        7
    );

    // By bare test name across suites.
    let report = run(&mut registry, Some("store_relation_resolves"));
    assert_eq!(report["totals"]["passed"], 1);
    assert_eq!(report["ok"], true);

    // No match is a usage error, not a pass.
    let report = run(&mut registry, Some("does-not-exist"));
    assert_eq!(report["ok"], false);
    assert!(report["error"]
        .as_str()
        .unwrap()
        .contains("no tests matched"));
}

#[test]
fn fixtures_never_touch_the_real_workspace() {
    let root = setup_testing_workspace("testing_isolation");
    let workspace = root.join("workspace");
    let mut registry = registry_for(&root);

    let report = run(&mut registry, None);
    assert_eq!(
        report["totals"]["failed"], 2,
        "only the intentional failures: {report}"
    );

    // The real workspace still has exactly its original file.
    assert!(workspace.join("lib/models/real.dart").exists());
    assert!(
        !workspace.join("out").exists(),
        "operation leaked into workspace"
    );
    assert!(
        !workspace.join("gen").exists(),
        "recipe leaked into workspace"
    );
    assert!(
        !workspace.join("lib/models/user.dart").exists(),
        "scaffold leaked"
    );
    // Temp-dir cleanup is covered by the unconditional remove_dir_all in
    // run_test_inner; a filesystem-level leak assertion is not deterministic
    // from a parallel test binary (sibling threads hold in-flight fixtures).
}

#[test]
fn no_suites_is_a_usage_error() {
    let root = temp_workspace("testing_empty");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("workspace")).unwrap();
    fs::create_dir_all(root.join(".codemod")).unwrap();
    let mut registry = registry_for(&root);
    let report = run(&mut registry, None);
    assert_eq!(report["ok"], false);
    assert!(report["error"]
        .as_str()
        .unwrap()
        .contains("no test suites found"));
}
