//! Luau asset load / classify / sandbox.

mod component;
mod error;
mod map;
mod query_lib;
mod recipe;
mod ops;
mod type_parse;
mod explorer;
mod shared;
mod stack;
mod table_util;
mod test_suite;
mod type_def;
mod variables;

pub use error::{Asset, AssetKind, LoadError, LoadResult};
pub use component::load_component_from_table;
pub use map::{load_map, load_map_from_table};
pub use query_lib::{load_query_lib, load_query_lib_from_table};
pub use recipe::{load_recipe, load_recipe_from_table};
pub use stack::load_stack_from_table;
pub use test_suite::load_test_suite_from_table;
pub use type_def::{load_type, load_type_from_table};
pub use variables::{load_variables, load_variables_from_table};

use mlua::{Lua, Table, Value};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use self::table_util::map_lua_error;

const MAX_INTERRUPTS: u64 = 50_000_000;

/// True for IDE-only declaration stubs (`.d.luau` / `.d.lua`).
pub fn is_declaration_file_name(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| name.ends_with(".d.luau") || name.ends_with(".d.lua"))
        .unwrap_or(false)
}

pub fn new_sandbox() -> Result<Lua, LoadError> {
    new_sandbox_inner(None)
}

/// Sandbox with `require` limited to Library modules under `codemod_root`.
pub fn new_sandbox_with_require(codemod_root: &Path) -> Result<Lua, LoadError> {
    new_sandbox_inner(Some(codemod_root))
}

fn new_sandbox_inner(codemod_root: Option<&Path>) -> Result<Lua, LoadError> {
    let lua = Lua::new();
    if let Some(root) = codemod_root {
        crate::require::install_codemod_require(&lua, root)?;
    }
    lua.sandbox(true)
        .map_err(|e| LoadError::Message(format!("failed to enable Luau sandbox: {e}")))?;
    let counter = Arc::new(AtomicU64::new(0));
    lua.set_interrupt(move |_| {
        let n = counter.fetch_add(1, Ordering::Relaxed);
        if n > MAX_INTERRUPTS {
            Err(mlua::Error::RuntimeError(
                "recipe exceeded instruction budget".to_string(),
            ))
        } else {
            Ok(mlua::VmState::Continue)
        }
    });
    Ok(lua)
}

pub fn eval_return_table(lua: &Lua, source: &str, chunk_name: &str) -> Result<Table, LoadError> {
    let chunk = lua
        .load(source)
        .set_name(chunk_name)
        .into_function()
        .map_err(|e| map_lua_error(e, source))?;
    let value: Value = chunk.call(()).map_err(|e| map_lua_error(e, source))?;
    match value {
        Value::Table(table) => Ok(table),
        other => Err(LoadError::Message(format!(
            "module must return a table, got {}",
            other.type_name()
        ))),
    }
}

pub fn classify_table(table: &Table) -> Result<AssetKind, LoadError> {
    let has_id = match table.get::<Value>("id") {
        Ok(Value::Nil) | Err(_) => false,
        Ok(_) => true,
    };
    let has_apply = match table.get::<Value>("apply") {
        Ok(Value::Function(_)) => true,
        Ok(Value::Nil) | Err(_) => false,
        Ok(other) => {
            return Err(LoadError::Message(format!(
                "apply must be a function, got {}",
                other.type_name()
            )));
        }
    };
    let has_parse = table
        .get::<Option<mlua::Function>>("parse")
        .map_err(|e| LoadError::Message(e.to_string()))?
        .is_some();

    if !has_id {
        if has_apply || has_parse {
            return Err(LoadError::Message(
                "asset requires top-level id (apply/parse without id is not a library module)"
                    .to_string(),
            ));
        }
        return Ok(AssetKind::Library);
    }

    if has_apply {
        return Ok(AssetKind::Recipe);
    }
    if has_parse {
        return Ok(AssetKind::Component);
    }
    if table
        .get::<Option<Table>>("components")
        .map_err(|e| LoadError::Message(e.to_string()))?
        .is_some()
    {
        return Ok(AssetKind::Stack);
    }
    if table
        .get::<Option<Table>>("map")
        .map_err(|e| LoadError::Message(e.to_string()))?
        .is_some()
    {
        return Ok(AssetKind::Map);
    }
    if table
        .get::<Option<Table>>("values")
        .map_err(|e| LoadError::Message(e.to_string()))?
        .is_some()
    {
        return Ok(AssetKind::Variables);
    }
    if table
        .get::<Option<Table>>("queries")
        .map_err(|e| LoadError::Message(e.to_string()))?
        .is_some()
    {
        return Ok(AssetKind::QueryLib);
    }
    if table
        .get::<Option<Table>>("fields")
        .map_err(|e| LoadError::Message(e.to_string()))?
        .is_some()
    {
        return Ok(AssetKind::Type);
    }
    if table
        .get::<Option<Table>>("tests")
        .map_err(|e| LoadError::Message(e.to_string()))?
        .is_some()
    {
        return Ok(AssetKind::TestSuite);
    }
    Ok(AssetKind::Unknown)
}

pub fn load_asset(source: &str, chunk_name: &str) -> Result<LoadResult, LoadError> {
    load_asset_with_root(source, chunk_name, None)
}

/// Load a catalog asset; when `codemod_root` is set, top-level `require` of Library helpers is allowed.
pub fn load_asset_with_root(
    source: &str,
    chunk_name: &str,
    codemod_root: Option<&Path>,
) -> Result<LoadResult, LoadError> {
    let lua = match codemod_root {
        Some(root) => new_sandbox_with_require(root)?,
        None => new_sandbox()?,
    };
    let table = eval_return_table(&lua, source, chunk_name)?;
    match classify_table(&table)? {
        AssetKind::Library => Ok(LoadResult::Library),
        AssetKind::Recipe => Ok(LoadResult::Asset(Box::new(Asset::Recipe(
            load_recipe_from_table(&table, source)?,
        )))),
        AssetKind::Map => Ok(LoadResult::Asset(Box::new(Asset::Map(
            load_map_from_table(&table)?,
        )))),
        AssetKind::Variables => Ok(LoadResult::Asset(Box::new(Asset::Variables(
            load_variables_from_table(&table)?,
        )))),
        AssetKind::QueryLib => Ok(LoadResult::Asset(Box::new(Asset::QueryLib(
            load_query_lib_from_table(&table)?,
        )))),
        AssetKind::Type => Ok(LoadResult::Asset(Box::new(Asset::Type(
            load_type_from_table(&table)?,
        )))),
        AssetKind::Stack => Ok(LoadResult::Asset(Box::new(Asset::Stack(
            load_stack_from_table(&table, source)?,
        )))),
        AssetKind::Component => Ok(LoadResult::Asset(Box::new(Asset::Component(
            load_component_from_table(&table, source)?,
        )))),
        AssetKind::TestSuite => Ok(LoadResult::Asset(Box::new(Asset::TestSuite(
            load_test_suite_from_table(&table, source)?,
        )))),
        AssetKind::Unknown => Err(LoadError::Message(
            "unknown asset shape: expected apply, parse (component), components (stack), map, values, queries, fields, or tests".to_string(),
        )),
    }
}

/// Classify a loaded module table for `require`: libraries are allowed; assets are rejected.
pub fn assert_library_module(table: &Table) -> Result<(), LoadError> {
    match classify_table(table)? {
        AssetKind::Library => Ok(()),
        kind => Err(LoadError::Message(format!(
            "cannot require {kind:?} module; use c:run / c:component / catalog lookup for assets (helpers must not have top-level id)"
        ))),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::input_kind::InputKind;

    #[test]
    fn loads_minimal_recipe() {
        let src = r#"
return {
  id = "demo.insert",
  args = { { name = "file", required = true, inputKind = "file" } },
  apply = function(c) end,
}
"#;
        let recipe = load_recipe(src, "demo.luau").unwrap();
        assert_eq!(recipe.id, "demo.insert");
        assert_eq!(recipe.args.len(), 1);
        assert!(recipe.args[0].required);
        assert_eq!(recipe.args[0].input_kind, InputKind::File);
    }

    #[test]
    fn rejects_missing_apply() {
        let src = r#"return { id = "x" }"#;
        assert!(load_recipe(src, "x.luau").is_err());
    }

    #[test]
    fn parses_list_input_kind() {
        let src = r#"
return {
  id = "demo.list",
  args = { { name = "properties", inputKind = { "ClassProperty" } } },
  apply = function(c) end,
}
"#;
        let recipe = load_recipe(src, "demo.luau").unwrap();
        assert_eq!(
            recipe.args[0].input_kind,
            InputKind::List(Box::new(InputKind::Type("ClassProperty".into())))
        );
    }

    #[test]
    fn rejects_nested_list_input_kind() {
        let src = r#"
return {
  id = "demo.bad",
  args = { { name = "x", inputKind = { { "text" } } } },
  apply = function(c) end,
}
"#;
        assert!(load_recipe(src, "x.luau").is_err());
    }

    #[test]
    fn loads_type_asset() {
        let src = r#"
return {
  id = "ClassProperty",
  fields = {
    { name = "type", inputKind = "choice", optionsFromMap = "column_type", required = true },
    { name = "name", inputKind = "text", required = true },
    { name = "initialValue", inputKind = "text" },
  },
}
"#;
        let def = load_type(src, "class_property.luau").unwrap();
        assert_eq!(def.id, "ClassProperty");
        assert_eq!(def.fields.len(), 3);
        assert_eq!(
            def.fields[0].options_from_map.as_deref(),
            Some("column_type")
        );
    }

    #[test]
    fn rejects_reserved_type_id() {
        let src = r#"return { id = "file", fields = { { name = "x" } } }"#;
        assert!(load_type(src, "t.luau").is_err());
    }

    #[test]
    fn aliases_symbol_to_text() {
        let src = r#"
return {
  id = "demo.sym",
  args = { { name = "className", inputKind = "symbol" } },
  apply = function(c) end,
}
"#;
        let recipe = load_recipe(src, "demo.luau").unwrap();
        assert_eq!(recipe.args[0].input_kind, InputKind::Text);
    }

    #[test]
    fn classify_table_recipe() {
        let src = r#"return { id = "x", apply = function(c) end }"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "x.luau").unwrap();
        assert_eq!(classify_table(&table).unwrap(), AssetKind::Recipe);
    }

    #[test]
    fn classify_table_map() {
        let src = r#"return { id = "m", map = { a = "b" } }"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "m.luau").unwrap();
        assert_eq!(classify_table(&table).unwrap(), AssetKind::Map);
    }

    #[test]
    fn classify_table_variables() {
        let src = r#"return { id = "v", values = { a = "b" } }"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "v.luau").unwrap();
        assert_eq!(classify_table(&table).unwrap(), AssetKind::Variables);
    }

    #[test]
    fn classify_table_query_lib() {
        let src = r#"return { id = "q", queries = { body = "(_) @x" } }"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "q.luau").unwrap();
        assert_eq!(classify_table(&table).unwrap(), AssetKind::QueryLib);
    }

    #[test]
    fn classify_table_type() {
        let src = r#"return { id = "T", fields = { { name = "x" } } }"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "t.luau").unwrap();
        assert_eq!(classify_table(&table).unwrap(), AssetKind::Type);
    }

    #[test]
    fn classify_table_unknown() {
        let src = r#"return { id = "x" }"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "x.luau").unwrap();
        assert_eq!(classify_table(&table).unwrap(), AssetKind::Unknown);
    }

    #[test]
    fn classify_table_apply_not_function_is_error() {
        let src = r#"return { id = "x", apply = "not a function" }"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "x.luau").unwrap();
        assert!(classify_table(&table).is_err());
    }

    /// Unwrap a `LoadResult` into its boxed `Asset` (derefed for matching).
    fn loaded_asset(result: LoadResult) -> Asset {
        match result {
            LoadResult::Asset(asset) => *asset,
            other => panic!("expected asset, got {other:?}"),
        }
    }

    #[test]
    fn load_asset_recipe() {
        let src = r#"
return {
  id = "demo.insert",
  apply = function(c) end,
}
"#;
        let asset = load_asset(src, "demo.luau").unwrap();
        assert!(matches!(loaded_asset(asset), Asset::Recipe(r) if r.id == "demo.insert"));
    }

    #[test]
    fn load_asset_map() {
        let src = r#"return { id = "m", map = { a = "b" } }"#;
        let asset = load_asset(src, "m.luau").unwrap();
        assert!(matches!(
            loaded_asset(asset),
            Asset::Map(m) if m.id == "m" && m.map["a"] == "b"
        ));
    }

    #[test]
    fn load_asset_unknown() {
        let src = r#"return { id = "x" }"#;
        let err = load_asset(src, "x.luau").unwrap_err().to_string();
        assert!(err.contains("unknown asset shape"));
    }

    #[test]
    fn classify_table_library() {
        let src = r#"return { contains = function(s, sub) return true end }"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "lib.luau").unwrap();
        assert_eq!(classify_table(&table).unwrap(), AssetKind::Library);
    }

    #[test]
    fn classify_table_apply_without_id_is_malformed() {
        let src = r#"return { apply = function(c) end }"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "bad.luau").unwrap();
        let err = classify_table(&table).unwrap_err().to_string();
        assert!(err.contains("requires top-level id"));
    }

    #[test]
    fn load_asset_library() {
        let src = r#"return { helper = function() return 1 end }"#;
        assert!(matches!(
            load_asset(src, "lib.luau").unwrap(),
            LoadResult::Library
        ));
    }

    #[test]
    fn classify_table_stack() {
        let src = r#"return { id = "s", components = { "a.b" } }"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "stack.luau").unwrap();
        assert_eq!(classify_table(&table).unwrap(), AssetKind::Stack);
    }

    #[test]
    fn classify_table_component() {
        let src = r#"
return {
  id = "example.entity",
  identify = function(c) return {} end,
  parse = function(c, l) return {} end,
  verify = function(c, i) return {} end,
}
"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "entity.luau").unwrap();
        assert_eq!(classify_table(&table).unwrap(), AssetKind::Component);
    }

    #[test]
    fn component_without_verify_classifies_as_component() {
        // v1.4: verify is optional — id + parse classify a component.
        let src = r#"
return {
  id = "example.entity",
  identify = function(c) return {} end,
  parse = function(c, l) return {} end,
}
"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "entity.luau").unwrap();
        assert_eq!(classify_table(&table).unwrap(), AssetKind::Component);
        match loaded_asset(load_asset(src, "entity.luau").unwrap()) {
            Asset::Component(c) => {
                assert_eq!(c.id, "example.entity");
                assert!(c.has_identify);
                assert!(!c.has_verify);
                assert!(!c.has_relate);
            }
            other => panic!("expected component, got {other:?}"),
        }
    }

    #[test]
    fn component_relate_detected_as_optional_hook() {
        let src = r#"
return {
  id = "example.store",
  parse = function(c, l) return {} end,
  relate = function(c, i, index) return nil end,
}
"#;
        match loaded_asset(load_asset(src, "store.luau").unwrap()) {
            Asset::Component(c) => {
                assert!(c.has_relate);
                assert!(!c.has_verify);
                assert!(!c.has_identify);
            }
            other => panic!("expected component, got {other:?}"),
        }
    }

    #[test]
    fn load_asset_stack_and_component() {
        let stack = r#"return { id = "stack", components = { "a.b" } }"#;
        let component = r#"
return {
  id = "a.b",
  identify = function(c) return {} end,
  parse = function(c, l) return {} end,
  verify = function(c, i) return {} end,
}
"#;
        assert!(matches!(
            loaded_asset(load_asset(stack, "stack.luau").unwrap()),
            Asset::Stack(s) if s.id == "stack"
        ));
        let loaded = load_asset(component, "a.b.luau").unwrap();
        assert!(matches!(
            loaded_asset(loaded),
            Asset::Component(c) if c.id == "a.b"
        ));
    }

    #[test]
    fn classify_table_test_suite() {
        let src = r#"
return {
  id = "flutter.model.tests",
  target = "flutter.model",
  tests = {
    parses_fields = function(t) end,
    relates_stores = function(t) end,
  },
}
"#;
        let lua = new_sandbox().unwrap();
        let table = eval_return_table(&lua, src, "flutter.model.test.luau").unwrap();
        assert_eq!(classify_table(&table).unwrap(), AssetKind::TestSuite);
        match loaded_asset(load_asset(src, "flutter.model.test.luau").unwrap()) {
            Asset::TestSuite(suite) => {
                assert_eq!(suite.id, "flutter.model.tests");
                assert_eq!(suite.target.as_deref(), Some("flutter.model"));
                assert_eq!(
                    suite.test_names,
                    vec!["parses_fields".to_string(), "relates_stores".to_string()]
                );
            }
            other => panic!("expected test suite, got {other:?}"),
        }
    }

    #[test]
    fn test_suite_rejects_non_function_test() {
        let src = r#"return { id = "x.tests", tests = { broken = "nope" } }"#;
        let err = load_asset(src, "x.test.luau").unwrap_err().to_string();
        assert!(err.contains("must be a function"), "got: {err}");
    }

    #[test]
    fn test_suite_rejects_empty_tests() {
        let src = r#"return { id = "x.tests", tests = {} }"#;
        let err = load_asset(src, "x.test.luau").unwrap_err().to_string();
        assert!(err.contains("at least one test"), "got: {err}");
    }

    #[test]
    fn load_asset_stack_with_codebase() {
        let stack = r#"return {
  id = "stack",
  components = { "a.b" },
  codebase = { "lib/**/*.dart", "src/*.rs" },
}"#;
        match loaded_asset(load_asset(stack, "stack.luau").unwrap()) {
            Asset::Stack(s) => {
                assert_eq!(s.id, "stack");
                assert_eq!(
                    s.codebase,
                    vec!["lib/**/*.dart".to_string(), "src/*.rs".to_string()]
                );
            }
            other => panic!("expected stack, got {other:?}"),
        }
    }

    #[test]
    fn load_asset_stack_with_relations() {
        let stack = r#"return {
  id = "stack",
  components = { "a.b" },
  relations = {
    stores = { layout = "edge" },
    lives_in = { layout = "member" },
  },
}"#;
        match loaded_asset(load_asset(stack, "stack.luau").unwrap()) {
            Asset::Stack(s) => {
                assert_eq!(s.relations["stores"].layout, "edge");
                assert_eq!(s.relations["lives_in"].layout, "member");
            }
            other => panic!("expected stack, got {other:?}"),
        }
    }

    #[test]
    fn load_asset_stack_with_zones_and_zone_roles() {
        let stack = r#"return {
  id = "stack",
  components = { "repo.zone" },
  zones = {
    { id = "structural.modules", role = "structural", title = "Module barrels" },
    { id = "docs.surface", role = "documentation", title = "Docs" },
    { id = "nested", role = "architecture", parentId = "structural.modules" },
  },
  zoneRoles = {
    structural = { ignoreRelations = true },
    noise = { ignoreCoverage = true, ignoreRelations = false },
  },
}"#;
        match loaded_asset(load_asset(stack, "stack.luau").unwrap()) {
            Asset::Stack(s) => {
                assert_eq!(s.zones.len(), 3);
                assert_eq!(s.zones[0].id, "structural.modules");
                assert_eq!(s.zones[0].role.as_deref(), Some("structural"));
                assert_eq!(s.zones[0].title.as_deref(), Some("Module barrels"));
                assert!(s.zones[0].parent_id.is_none());
                assert_eq!(s.zones[2].parent_id.as_deref(), Some("structural.modules"));
                let structural = s.zone_roles.get("structural").expect("structural role");
                assert!(structural.ignore_relations);
                assert!(!structural.ignore_coverage);
                let noise = s.zone_roles.get("noise").expect("noise role");
                assert!(noise.ignore_coverage);
                assert!(!noise.ignore_relations);
                assert!(!s.zone_roles.contains_key("documentation"));
            }
            other => panic!("expected stack, got {other:?}"),
        }
    }
}
