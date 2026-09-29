//! Fixture-based test runner for `*.test.luau` suites.
//!
//! Each test runs in a fresh Luau sandbox with a test context `t`:
//! `t:workspace(spec)` materializes a fixture tree into a temp dir (optional
//! `base` dir copied first, inline `files` written over it) and returns a
//! fixture-bound host `ws` — a `RecipeRegistry` rooted at the temp dir with the
//! real `.codemod` root, so all production machinery (scan, hooks, operations,
//! recipe apply) runs unchanged against the fixture.
//!
//! Assertions are plain Luau (`assert` / `error`); the `t:assert*` helpers live
//! in a Luau bootstrap chunk so failures carry the test's own line number via
//! `error(msg, 2)` — the host parses the location out of the error message.
//! Mutations (`ws:apply` / `ws:scaffold` / `ws:applyRecipe`) flush to the temp
//! dir only; the real workspace is never touched.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

use codemod_recipe_core::apply::atomic_apply::apply_operations_atomically;
use codemod_recipe_engine::{LanguageRegistry, RegistryConfig};
use codemod_recipe_lua::json_lua::{json_to_lua, lua_to_json};
use codemod_recipe_lua::model::TestSuiteDef;
use codemod_recipe_lua::{eval_return_table, new_sandbox_with_require, require_chunk_name};
use mlua::{Function, Lua, Table, Value};
use serde::Serialize;

use crate::component::{
    component_scan, invoke_hook, invoke_operation, invoke_scaffold, ComponentHookRequest,
    ComponentInvokeRequest, ComponentScaffoldRequest,
};
use crate::engine::runner::{collect_recipe_changes, resolve_recipe};
use crate::host_api::registry::{rt, set_fn};
use crate::lua_runtime::RunKind;
use crate::preview::diff::build_file_preview_from_change;
use crate::preview::patch_selector::apply_changes_with_selection;
use crate::recipe::registry::RecipeRegistry;
use crate::sandbox::path_sandbox::PathSandbox;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestRunReport {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub suites: Vec<TestSuiteReport>,
    pub totals: TestTotals,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestSuiteReport {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    pub tests: Vec<TestCaseReport>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestCaseReport {
    pub name: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TestTotals {
    pub passed: usize,
    pub failed: usize,
}

/// Luau bootstrap for the test context `t`. Assertions live in Luau (not mlua
/// closures) so `error(msg, 2)` points at the calling test line and the helpers
/// stay readable/hackable — same pattern as the relations graph facade.
const TEST_CONTEXT_LUA: &str = r#"
local M = {}

local function render(value, seen)
	local t = type(value)
	if t == "string" then
		return string.format("%q", value)
	end
	if t == "table" then
		seen = seen or {}
		if seen[value] then
			return "<cycle>"
		end
		seen[value] = true
		local parts = {}
		for k, v in pairs(value) do
			parts[#parts + 1] = tostring(k) .. " = " .. render(v, seen)
		end
		seen[value] = nil
		table.sort(parts)
		return "{ " .. table.concat(parts, ", ") .. " }"
	end
	return tostring(value)
end

local function deep_eq(a, b)
	if a == b then
		return true
	end
	if type(a) ~= "table" or type(b) ~= "table" then
		return false
	end
	for k, v in pairs(a) do
		if not deep_eq(v, b[k]) then
			return false
		end
	end
	for k in pairs(b) do
		if a[k] == nil then
			return false
		end
	end
	return true
end

function M.build(prims)
	local t = {}
	function t:workspace(spec)
		return prims.workspace(spec)
	end
	function t:assert(v, msg)
		if not v then
			error(msg or "assertion failed", 2)
		end
		return v
	end
	function t:assertEq(got, want, msg)
		if not deep_eq(got, want) then
			local prefix = msg and (msg .. ": ") or ""
			error(prefix .. "expected " .. render(want) .. ", got " .. render(got), 2)
		end
		return got
	end
	function t:fail(msg)
		error(msg or "test failed", 2)
	end
	return t
end

return M
"#;

struct FixtureState {
    root: PathBuf,
    registry: RecipeRegistry,
}

/// Fixture registries created by `t:workspace` within one test; `ws` tables
/// hold an index into this store. Dropped (and temp dirs removed) after the test.
type FixtureStore = Rc<RefCell<Vec<FixtureState>>>;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn run_tests(registry: &RecipeRegistry, filter: Option<&str>) -> TestRunReport {
    let mut suites = Vec::new();
    let mut passed = 0;
    let mut failed = 0;
    for suite in registry.test_suites().values() {
        let selected: Vec<&String> = suite
            .test_names
            .iter()
            .filter(|name| test_selected(suite, name, filter))
            .collect();
        if selected.is_empty() {
            continue;
        }
        let mut tests = Vec::new();
        for name in selected {
            let report = run_single_test(registry, suite, name);
            if report.ok {
                passed += 1;
            } else {
                failed += 1;
            }
            tests.push(report);
        }
        suites.push(TestSuiteReport {
            id: suite.id.clone(),
            path: suite.path.as_ref().map(|p| p.display().to_string()),
            tests,
        });
    }
    let totals = TestTotals { passed, failed };
    if suites.is_empty() {
        let error = match filter {
            Some(f) => format!(
                "no tests matched filter '{f}' (suites: {})",
                registry
                    .test_suites()
                    .keys()
                    .cloned()
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            None => {
                "no test suites found (add *.test.luau modules with id + tests; check validate_recipes for load errors)".to_string()
            }
        };
        return TestRunReport {
            ok: false,
            error: Some(error),
            suites,
            totals,
        };
    }
    TestRunReport {
        ok: failed == 0,
        error: None,
        suites,
        totals,
    }
}

/// Filter matches suite id, suite `target`, `suite/test` (split on `/`), or a
/// bare test name substring.
fn test_selected(suite: &TestSuiteDef, test: &str, filter: Option<&str>) -> bool {
    let Some(f) = filter else { return true };
    let f = f.trim();
    if f.is_empty() {
        return true;
    }
    if let Some((suite_part, test_part)) = f.split_once('/') {
        return suite.id.contains(suite_part) && test.contains(test_part);
    }
    suite.id.contains(f)
        || suite.target.as_deref().is_some_and(|t| t.contains(f))
        || test.contains(f)
}

fn run_single_test(
    registry: &RecipeRegistry,
    suite: &TestSuiteDef,
    test_name: &str,
) -> TestCaseReport {
    match run_test_inner(registry, suite, test_name) {
        Ok(()) => TestCaseReport {
            name: test_name.to_string(),
            ok: true,
            error: None,
            line: None,
        },
        Err(message) => {
            let line = parse_error_line(&message);
            TestCaseReport {
                name: test_name.to_string(),
                ok: false,
                error: Some(clean_error_message(&message)),
                line,
            }
        }
    }
}

fn run_test_inner(
    registry: &RecipeRegistry,
    suite: &TestSuiteDef,
    test_name: &str,
) -> Result<(), String> {
    let codemod_root = registry.codemod_root().to_path_buf();
    let lua = new_sandbox_with_require(&codemod_root).map_err(|e| e.to_string())?;
    let chunk_name = suite
        .path
        .as_ref()
        .map(|p| require_chunk_name(p))
        .unwrap_or_else(|| suite.id.clone());
    let module = eval_return_table(&lua, &suite.source, &chunk_name).map_err(|e| e.to_string())?;
    let tests: Table = module.get("tests").map_err(|e| e.to_string())?;
    let func: Function = tests.get(test_name).map_err(|e| e.to_string())?;
    let store: FixtureStore = Rc::new(RefCell::new(Vec::new()));
    let t = build_test_context(
        &lua,
        Rc::clone(&store),
        codemod_root,
        registry.language_registry.config().clone(),
    )?;
    let result = func.call::<()>(t).map_err(|e| e.to_string());
    for state in store.borrow().iter() {
        let _ = std::fs::remove_dir_all(&state.root);
    }
    result
}

/// Luau error messages carry the location as `chunk:line: message` (chunk either
/// a plain path or `[string "name"]`). mlua prefixes `runtime error: ` and may
/// append a stack traceback. Lift the first `:<digits>:` — the location always
/// precedes the message body.
fn parse_error_line(message: &str) -> Option<u32> {
    for (idx, _) in message.match_indices(':') {
        let rest = &message[idx + 1..];
        let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
        if !digits.is_empty() && rest[digits.len()..].starts_with(':') {
            return digits.parse().ok();
        }
    }
    None
}

/// Drop the noisy `stack traceback:` tail — the report carries the line separately.
fn clean_error_message(message: &str) -> String {
    message
        .split("\nstack traceback:")
        .next()
        .unwrap_or(message)
        .trim_end()
        .to_string()
}

fn build_test_context(
    lua: &Lua,
    store: FixtureStore,
    codemod_root: PathBuf,
    language_config: RegistryConfig,
) -> Result<Table, String> {
    let prims = lua.create_table().map_err(|e| e.to_string())?;
    set_fn(lua, &prims, "workspace", move |lua, spec: Value| {
        let root = materialize_workspace(&codemod_root, &spec).map_err(rt)?;
        let mut fixture_registry = RecipeRegistry::new(root.clone(), codemod_root.clone());
        fixture_registry.language_registry = LanguageRegistry::with_config(language_config.clone());
        fixture_registry.reload();
        let mut store_mut = store.borrow_mut();
        store_mut.push(FixtureState {
            root,
            registry: fixture_registry,
        });
        let idx = store_mut.len() - 1;
        drop(store_mut);
        build_ws(lua, Rc::clone(&store), idx)
    })?;
    let module =
        eval_return_table(lua, TEST_CONTEXT_LUA, "test_context").map_err(|e| e.to_string())?;
    let build: Function = module.get("build").map_err(|e| e.to_string())?;
    build.call::<Table>(prims).map_err(|e| e.to_string())
}

/// Parse `t:workspace(spec)`: a fixture dir string, or `{ base?, files? }`.
/// Returns `(base_dir, inline_files)`.
fn parse_workspace_spec(
    spec: &Value,
) -> Result<(Option<String>, BTreeMap<String, String>), String> {
    match spec {
        Value::String(s) => Ok((
            Some(s.to_str().map_err(|e| e.to_string())?.to_string()),
            BTreeMap::new(),
        )),
        Value::Table(t) => {
            let base = match t.get::<Value>("base").map_err(|e| e.to_string())? {
                Value::Nil => None,
                Value::String(s) => Some(s.to_str().map_err(|e| e.to_string())?.to_string()),
                other => {
                    return Err(format!(
                        "workspace base must be a string, got {}",
                        other.type_name()
                    ))
                }
            };
            let mut files = BTreeMap::new();
            if let Value::Table(files_tbl) = t.get::<Value>("files").map_err(|e| e.to_string())? {
                for pair in files_tbl.pairs::<String, String>() {
                    let (path, content) = pair.map_err(|e| e.to_string())?;
                    files.insert(path, content);
                }
            }
            // An empty table is deliberate: an empty workspace (scaffold tests).
            Ok((base, files))
        }
        other => Err(format!(
            "workspace spec must be a fixture dir string or a table, got {}",
            other.type_name()
        )),
    }
}

fn materialize_workspace(codemod_root: &Path, spec: &Value) -> Result<PathBuf, String> {
    let (base, files) = parse_workspace_spec(spec)?;
    let root = fresh_temp_root()?;
    // On any materialization error the dir is untracked — remove it immediately.
    let result = (|| -> Result<(), String> {
        if let Some(base) = base {
            let base_dir = resolve_fixture_base(codemod_root, &base)?;
            copy_dir_all(&base_dir, &root)?;
        }
        for (relative, content) in files {
            write_inline_file(&root, &relative, &content)?;
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_dir_all(&root);
    }
    result.map(|_| root)
}

fn fresh_temp_root() -> Result<PathBuf, String> {
    let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("codemod_test_{}_{n}", std::process::id()));
    std::fs::create_dir_all(&root).map_err(|e| format!("failed to create temp workspace: {e}"))?;
    Ok(root)
}

/// Fixture dirs live under `.codemod/tests/`; `base` is relative to it and must
/// not escape (no absolute paths, no `..`).
fn resolve_fixture_base(codemod_root: &Path, base: &str) -> Result<PathBuf, String> {
    let rel = Path::new(base);
    if rel.is_absolute() || rel.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(format!(
            "fixture base must be a relative path inside .codemod/tests: {base}"
        ));
    }
    let dir = codemod_root.join("tests").join(rel);
    if !dir.is_dir() {
        return Err(format!(
            "fixture dir not found: {base} (expected under .codemod/tests/)"
        ));
    }
    Ok(dir)
}

fn write_inline_file(root: &Path, relative: &str, content: &str) -> Result<(), String> {
    let rel = Path::new(relative);
    if rel.is_absolute() || rel.components().any(|c| matches!(c, Component::ParentDir)) {
        return Err(format!(
            "inline fixture path must be workspace-relative (no '..'): {relative}"
        ));
    }
    let absolute = root.join(rel);
    if let Some(parent) = absolute.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(&absolute, content).map_err(|e| format!("failed to write {relative}: {e}"))
}

fn copy_dir_all(src: &Path, dst: &Path) -> Result<(), String> {
    for entry in std::fs::read_dir(src).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_type = entry.file_type().map_err(|e| e.to_string())?;
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            copy_dir_all(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn lua_args_to_map(args: &Table) -> Result<BTreeMap<String, serde_json::Value>, String> {
    let mut out = BTreeMap::new();
    for pair in args.pairs::<String, Value>() {
        let (key, value) = pair.map_err(|e| e.to_string())?;
        out.insert(key, lua_to_json(&value)?);
    }
    Ok(out)
}

fn response_to_lua(lua: &Lua, value: impl Serialize) -> Result<Value, mlua::Error> {
    let json = serde_json::to_value(value).map_err(|e| rt(e.to_string()))?;
    json_to_lua(lua, &json).map_err(rt)
}

/// Build the fixture-bound `ws` table: thin wrappers over the production
/// scan/hook/operation/recipe machinery running on the fixture registry.
fn build_ws(lua: &Lua, store: FixtureStore, idx: usize) -> Result<Table, mlua::Error> {
    let ws = lua.create_table()?;

    {
        let store = Rc::clone(&store);
        ws.set(
            "scan",
            lua.create_function(move |lua, _this: Table| {
                let mut store = store.borrow_mut();
                let fixture = &mut store[idx];
                let resp = component_scan(&mut fixture.registry);
                if !resp.ok {
                    return Err(rt(resp
                        .error
                        .unwrap_or_else(|| "component scan failed".to_string())));
                }
                response_to_lua(lua, resp)
            })?,
        )?;
    }

    // Read hooks (identify/parse/verify/relate) share one path: invoke_hook in
    // Preview kind; verify/relate scan on demand when no cache exists yet.
    for hook in ["identify", "parse", "verify", "relate"] {
        let store = Rc::clone(&store);
        ws.set(
            hook,
            lua.create_function(
                move |lua, (_this, component_id, arg): (Table, String, Value)| {
                    let json_arg = lua_to_json(&arg).map_err(rt)?;
                    let (locator, instance) = match hook {
                        "parse" => (Some(json_arg), None),
                        "identify" => (None, None),
                        _ => (None, Some(json_arg)),
                    };
                    let mut store = store.borrow_mut();
                    let fixture = &mut store[idx];
                    let resp = invoke_hook(
                        &mut fixture.registry,
                        ComponentHookRequest {
                            component_id: component_id.clone(),
                            hook: hook.to_string(),
                            kind: RunKind::Preview,
                            locator,
                            instance,
                            operation: None,
                            args: BTreeMap::new(),
                        },
                    );
                    if !resp.ok {
                        return Err(rt(resp.error.unwrap_or_else(|| format!("{hook} failed"))));
                    }
                    response_to_lua(lua, resp.result.unwrap_or(serde_json::Value::Null))
                },
            )?,
        )?;
    }

    {
        let store = Rc::clone(&store);
        ws.set(
            "apply",
            lua.create_function(
                // componentId is accepted for call-site readability; the
                // instance id already embeds it (`component/id`).
                move |lua,
                      (_this, _component_id, operation, args, instance_id): (
                    Table,
                    String,
                    String,
                    Table,
                    String,
                )| {
                    let args = lua_args_to_map(&args).map_err(rt)?;
                    let mut store = store.borrow_mut();
                    let fixture = &mut store[idx];
                    // Apply kind flushes to the fixture temp dir, so rescan /
                    // readFile observe the mutation. The real workspace is
                    // never touched (sandbox is bound to the temp root).
                    let resp = invoke_operation(
                        &mut fixture.registry,
                        ComponentInvokeRequest {
                            instance_id,
                            operation,
                            args,
                            kind: RunKind::Apply,
                        },
                    );
                    if !resp.ok {
                        return Err(rt(resp
                            .error
                            .unwrap_or_else(|| "operation failed".to_string())));
                    }
                    response_to_lua(lua, resp)
                },
            )?,
        )?;
    }

    {
        let store = Rc::clone(&store);
        ws.set(
            "scaffold",
            lua.create_function(
                move |lua, (_this, component_id, args): (Table, String, Table)| {
                    let args = lua_args_to_map(&args).map_err(rt)?;
                    let mut store = store.borrow_mut();
                    let fixture = &mut store[idx];
                    let resp = invoke_scaffold(
                        &mut fixture.registry,
                        ComponentScaffoldRequest {
                            component_id,
                            args,
                            kind: RunKind::Apply,
                        },
                    );
                    if !resp.ok {
                        return Err(rt(resp
                            .error
                            .unwrap_or_else(|| "scaffold failed".to_string())));
                    }
                    response_to_lua(lua, resp)
                },
            )?,
        )?;
    }

    {
        let store = Rc::clone(&store);
        ws.set(
            "applyRecipe",
            lua.create_function(
                move |lua, (_this, recipe_id, args): (Table, String, Table)| {
                    let args = lua_args_to_map(&args).map_err(rt)?;
                    let mut store = store.borrow_mut();
                    let fixture = &mut store[idx];
                    let (recipe, recipe_path) =
                        resolve_recipe(&fixture.registry, Some(&recipe_id), None).map_err(rt)?;
                    // Preview collect: postExec scripts are flagged, never executed.
                    let collected = collect_recipe_changes(
                        &fixture.registry,
                        &recipe,
                        recipe_path.as_deref(),
                        &args,
                        RunKind::Preview,
                    )
                    .map_err(|e| rt(e.message))?;
                    let mut files = Vec::new();
                    for change in &collected.changes {
                        if change.is_skipped() {
                            continue;
                        }
                        files.push(
                            build_file_preview_from_change(change, true, None)
                                .map_err(|e| rt(e.to_string()))?,
                        );
                    }
                    // Flush file changes to the fixture temp dir (production
                    // write path) so readFile/scan observe them.
                    let workspace = fixture.registry.workspace_root.clone();
                    let applied = apply_changes_with_selection(
                        |relative| {
                            PathSandbox::new(workspace.clone())
                                .resolve_workspace_relative(relative)
                                .map_err(|e| e.message)
                        },
                        &collected.changes,
                        &BTreeMap::new(),
                    )
                    .map_err(rt)?;
                    let ops: Vec<_> = applied.iter().map(|c| c.operation.clone()).collect();
                    apply_operations_atomically(&ops).map_err(rt)?;
                    let applied_paths: Vec<String> =
                        applied.iter().map(|c| c.relative_path.clone()).collect();
                    response_to_lua(
                        lua,
                        serde_json::json!({
                            "files": files,
                            "applied": applied_paths,
                            "transcript": collected.transcript,
                            "sawPostExec": collected.saw_post_exec,
                        }),
                    )
                },
            )?,
        )?;
    }

    {
        let store = Rc::clone(&store);
        ws.set(
            "readFile",
            lua.create_function(move |lua, (_this, path): (Table, String)| {
                let store = store.borrow();
                let fixture = &store[idx];
                let sandbox = PathSandbox::new(fixture.registry.workspace_root.clone());
                let absolute = sandbox
                    .resolve_workspace_relative(&path)
                    .map_err(|e| rt(e.message))?;
                if !absolute.exists() {
                    return Ok(Value::Nil);
                }
                let content = std::fs::read_to_string(&absolute)
                    .map_err(|e| rt(format!("failed to read {path}: {e}")))?;
                Ok(Value::String(lua.create_string(&content)?))
            })?,
        )?;
    }

    Ok(ws)
}
