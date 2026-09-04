use mlua::{Lua, Table, Value};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;

use crate::input_kind::InputKind;
use crate::model::{
    Arg, ArgFrom, ArgFromSpec, ComponentDef, ExplorerMenuEntry, ExplorerMenuKind, MapAsset,
    QueryLibAsset, Recipe, StackDef, TypeDef, VariablesAsset,
};

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("{0}")]
    Message(String),
    #[error("{message}")]
    Located {
        message: String,
        line: Option<u32>,
        column: Option<u32>,
    },
}

impl LoadError {
    pub fn location(&self) -> Option<(u32, u32)> {
        match self {
            Self::Located {
                line: Some(line),
                column: Some(column),
                ..
            } => Some((*line, *column)),
            _ => None,
        }
    }
}

impl From<mlua::Error> for LoadError {
    fn from(err: mlua::Error) -> Self {
        map_lua_error(err, "")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Recipe,
    Map,
    Variables,
    QueryLib,
    Type,
    Stack,
    Component,
    Unknown,
}

#[derive(Debug)]
pub enum Asset {
    Recipe(Recipe),
    Map(MapAsset),
    Variables(VariablesAsset),
    QueryLib(QueryLibAsset),
    Type(TypeDef),
    Stack(StackDef),
    Component(ComponentDef),
}

const MAX_INTERRUPTS: u64 = 50_000_000;

pub fn new_sandbox() -> Result<Lua, LoadError> {
    let lua = Lua::new();
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
    match table.get::<Value>("apply") {
        Ok(Value::Function(_)) => return Ok(AssetKind::Recipe),
        Ok(Value::Nil) | Err(_) => {}
        Ok(other) => {
            return Err(LoadError::Message(format!(
                "apply must be a function, got {}",
                other.type_name()
            )));
        }
    }
    let has_parse = table
        .get::<Option<mlua::Function>>("parse")
        .map_err(|e| LoadError::Message(e.to_string()))?
        .is_some();
    let has_verify = table
        .get::<Option<mlua::Function>>("verify")
        .map_err(|e| LoadError::Message(e.to_string()))?
        .is_some();
    if has_parse {
        if has_verify {
            return Ok(AssetKind::Component);
        }
        return Err(LoadError::Message(
            "component requires verify when parse is present".to_string(),
        ));
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
    Ok(AssetKind::Unknown)
}

pub fn load_asset(source: &str, chunk_name: &str) -> Result<Asset, LoadError> {
    let lua = new_sandbox()?;
    let table = eval_return_table(&lua, source, chunk_name)?;
    match classify_table(&table)? {
        AssetKind::Recipe => Ok(Asset::Recipe(load_recipe_from_table(&table, source)?)),
        AssetKind::Map => Ok(Asset::Map(load_map_from_table(&table)?)),
        AssetKind::Variables => Ok(Asset::Variables(load_variables_from_table(&table)?)),
        AssetKind::QueryLib => Ok(Asset::QueryLib(load_query_lib_from_table(&table)?)),
        AssetKind::Type => Ok(Asset::Type(load_type_from_table(&table)?)),
        AssetKind::Stack => Ok(Asset::Stack(load_stack_from_table(&table, source)?)),
        AssetKind::Component => Ok(Asset::Component(load_component_from_table(&table, source)?)),
        AssetKind::Unknown => Err(LoadError::Message(
            "unknown asset shape: expected apply, parse+verify (component), components (stack), map, values, queries, or fields".to_string(),
        )),
    }
}

pub fn load_recipe(source: &str, chunk_name: &str) -> Result<Recipe, LoadError> {
    let lua = new_sandbox()?;
    let table = eval_return_table(&lua, source, chunk_name)?;
    load_recipe_from_table(&table, source)
}

pub fn load_recipe_from_table(table: &Table, source: &str) -> Result<Recipe, LoadError> {
    let mut recipe = recipe_from_table(table)?;
    recipe.source = source.to_string();
    if recipe.id.trim().is_empty() {
        return Err(LoadError::Message("recipe id is required".to_string()));
    }
    table
        .get::<Option<mlua::Function>>("apply")
        .map_err(|e| LoadError::Message(e.to_string()))?
        .ok_or_else(|| LoadError::Message("recipe apply function is required".to_string()))?;
    Ok(recipe)
}

pub fn load_map(source: &str, chunk_name: &str) -> Result<MapAsset, LoadError> {
    let lua = new_sandbox()?;
    let table = eval_return_table(&lua, source, chunk_name)?;
    load_map_from_table(&table)
}

pub fn load_map_from_table(table: &Table) -> Result<MapAsset, LoadError> {
    let id: String = required_string(table, "id")?;
    let description: Option<String> = optional_string(table, "description")?;
    let map_tbl: Table = table
        .get("map")
        .map_err(|_| LoadError::Message("map asset requires map = { ... }".to_string()))?;
    Ok(MapAsset {
        id,
        description,
        map: string_map(&map_tbl)?,
    })
}

pub fn load_variables(source: &str, chunk_name: &str) -> Result<VariablesAsset, LoadError> {
    let lua = new_sandbox()?;
    let table = eval_return_table(&lua, source, chunk_name)?;
    load_variables_from_table(&table)
}

pub fn load_variables_from_table(table: &Table) -> Result<VariablesAsset, LoadError> {
    let id: String = required_string(table, "id")?;
    let description: Option<String> = optional_string(table, "description")?;
    let values_tbl: Table = table
        .get("values")
        .map_err(|_| LoadError::Message("variables asset requires values = { ... }".to_string()))?;
    Ok(VariablesAsset {
        id,
        description,
        values: string_map(&values_tbl)?,
    })
}

pub fn load_query_lib(source: &str, chunk_name: &str) -> Result<QueryLibAsset, LoadError> {
    let lua = new_sandbox()?;
    let table = eval_return_table(&lua, source, chunk_name)?;
    load_query_lib_from_table(&table)
}

pub fn load_query_lib_from_table(table: &Table) -> Result<QueryLibAsset, LoadError> {
    let id: String = required_string(table, "id")?;
    let queries_tbl: Table = table
        .get("queries")
        .map_err(|_| LoadError::Message("query lib requires queries = { ... }".to_string()))?;
    Ok(QueryLibAsset {
        id,
        queries: string_map(&queries_tbl)?,
    })
}

pub fn load_type(source: &str, chunk_name: &str) -> Result<TypeDef, LoadError> {
    let lua = new_sandbox()?;
    let table = eval_return_table(&lua, source, chunk_name)?;
    load_type_from_table(&table)
}

pub fn load_type_from_table(table: &Table) -> Result<TypeDef, LoadError> {
    type_from_table(table, None)
}

pub fn load_stack_from_table(table: &Table, source: &str) -> Result<StackDef, LoadError> {
    let id = required_string(table, "id")?;
    let description = optional_string(table, "description")?;
    let components_tbl: Table = table.get("components").map_err(|_| {
        LoadError::Message("stack requires components = { ... }".to_string())
    })?;
    let components = string_array(&components_tbl)?;
    if components.is_empty() {
        return Err(LoadError::Message(
            "stack components must list at least one component id".to_string(),
        ));
    }
    Ok(StackDef {
        id,
        description,
        components,
        source: source.to_string(),
        path: None,
    })
}

pub fn load_component_from_table(table: &Table, source: &str) -> Result<ComponentDef, LoadError> {
    let id = required_string(table, "id")?;
    if id.trim().is_empty() {
        return Err(LoadError::Message("component id is required".to_string()));
    }
    let description = optional_string(table, "description")?;
    require_function(table, "parse")?;
    require_function(table, "verify")?;
    let operations = parse_operations(table)?;
    let has_scaffold = parse_scaffold(table)?;
    Ok(ComponentDef {
        id,
        description,
        operations,
        has_scaffold,
        source: source.to_string(),
        path: None,
    })
}

fn parse_scaffold(table: &Table) -> Result<bool, LoadError> {
    let Ok(scaffold): Result<Table, _> = table.get("scaffold") else {
        return Ok(false);
    };
    match scaffold.get::<Value>("apply") {
        Ok(Value::Function(_)) => Ok(true),
        Ok(other) => Err(LoadError::Message(format!(
            "scaffold requires apply function, got {}",
            other.type_name()
        ))),
        Err(e) => Err(LoadError::Message(format!(
            "scaffold requires apply function: {e}"
        ))),
    }
}

fn require_function(table: &Table, key: &str) -> Result<(), LoadError> {
    match table.get::<Value>(key) {
        Ok(Value::Function(_)) => Ok(()),
        Ok(other) => Err(LoadError::Message(format!(
            "component requires {key} function, got {}",
            other.type_name()
        ))),
        Err(e) => Err(LoadError::Message(format!(
            "component requires {key} function: {e}"
        ))),
    }
}

fn string_array(table: &Table) -> Result<Vec<String>, LoadError> {
    let mut out = Vec::new();
    for value in table.sequence_values::<Value>() {
        match value.map_err(|e| LoadError::Message(e.to_string()))? {
            Value::String(s) => out.push(s.to_str()?.to_string()),
            other => {
                return Err(LoadError::Message(format!(
                    "expected string in array, got {}",
                    other.type_name()
                )));
            }
        }
    }
    Ok(out)
}

fn parse_operations(table: &Table) -> Result<Vec<String>, LoadError> {
    let Ok(ops): Result<Table, _> = table.get("operations") else {
        return Ok(vec![]);
    };
    let mut names = Vec::new();
    for pair in ops.pairs::<Value, Table>() {
        let (key, op_tbl) = pair.map_err(|e| LoadError::Message(e.to_string()))?;
        let name = match key {
            Value::String(s) => s.to_str()?.to_string(),
            other => {
                return Err(LoadError::Message(format!(
                    "operation name must be string, got {}",
                    other.type_name()
                )));
            }
        };
        match op_tbl.get::<Value>("apply") {
            Ok(Value::Function(_)) => {}
            Ok(other) => {
                return Err(LoadError::Message(format!(
                    "operation {name} requires apply function, got {}",
                    other.type_name()
                )));
            }
            Err(e) => {
                return Err(LoadError::Message(format!(
                    "operation {name} requires apply function: {e}"
                )));
            }
        }
        names.push(name);
    }
    names.sort();
    Ok(names)
}

fn recipe_from_table(table: &Table) -> Result<Recipe, LoadError> {
    let id = required_string(table, "id")?;
    let name = optional_string(table, "name")?;
    let description = optional_string(table, "description")?;
    let args = parse_args(table)?;
    let maps = optional_nested_maps(table, "maps")?;
    let types = optional_types(table)?;
    let queries = optional_string_map_field(table, "queries")?;
    let post_execution = optional_string_list(table, "postExecution")?;
    let explorer_menu = parse_explorer_menu(table)?;
    Ok(Recipe {
        id,
        name,
        description,
        args,
        maps,
        types,
        queries,
        post_execution,
        explorer_menu,
        source: String::new(),
        path: None,
    })
}

fn parse_args(table: &Table) -> Result<Vec<Arg>, LoadError> {
    let Ok(seq): Result<Table, _> = table.get("args") else {
        return Ok(vec![]);
    };
    let mut args = Vec::new();
    for pair in seq.sequence_values::<Table>() {
        let row = pair.map_err(|e| LoadError::Message(e.to_string()))?;
        args.push(parse_arg(&row)?);
    }
    Ok(args)
}

fn parse_arg(row: &Table) -> Result<Arg, LoadError> {
    let input_kind = parse_input_kind(row)?;
    if matches!(input_kind, InputKind::Choice)
        && optional_string_list(row, "options")?.is_empty()
        && optional_string(row, "optionsFromMap")?.is_none()
    {
        return Err(LoadError::Message(
            "choice inputKind requires options or optionsFromMap".to_string(),
        ));
    }
    Ok(Arg {
        name: required_string(row, "name")?,
        required: optional_bool(row, "required")?.unwrap_or(false),
        input_kind,
        abbr: optional_string(row, "abbr")?,
        help: optional_string(row, "help")?,
        defaults_to: optional_string(row, "defaultsTo")?,
        options: optional_string_list(row, "options")?,
        options_from_map: optional_string(row, "optionsFromMap")?,
        allow_custom_value: optional_bool(row, "allowCustomValue")?,
        from: parse_arg_from(row)?,
    })
}

fn parse_input_kind(row: &Table) -> Result<InputKind, LoadError> {
    match row.get::<Value>("inputKind") {
        Ok(Value::Nil) | Err(_) => Ok(InputKind::Text),
        Ok(Value::String(s)) => Ok(InputKind::from_name(&s.to_str()?)),
        Ok(Value::Table(t)) => {
            let mut items = Vec::new();
            for v in t.sequence_values::<Value>() {
                items.push(v.map_err(|e| LoadError::Message(e.to_string()))?);
            }
            if items.len() != 1 {
                return Err(LoadError::Message(
                    "inputKind list must contain exactly one type name".to_string(),
                ));
            }
            match &items[0] {
                Value::String(s) => Ok(InputKind::List(Box::new(InputKind::from_name(
                    &s.to_str()?,
                )))),
                Value::Table(_) => Err(LoadError::Message(
                    "nested list inputKind ([[T]]) is not supported".to_string(),
                )),
                other => Err(LoadError::Message(format!(
                    "inputKind list item must be a string, got {}",
                    other.type_name()
                ))),
            }
        }
        Ok(other) => Err(LoadError::Message(format!(
            "inputKind must be a string or {{ \"Type\" }}, got {}",
            other.type_name()
        ))),
    }
}

fn type_from_table(table: &Table, id_override: Option<String>) -> Result<TypeDef, LoadError> {
    let id = match id_override {
        Some(id) => id,
        None => required_string(table, "id")?,
    };
    if InputKind::is_reserved_id(&id) {
        return Err(LoadError::Message(format!(
            "type id '{id}' is reserved for a builtin inputKind"
        )));
    }
    let description = optional_string(table, "description")?;
    let fields = parse_fields(table)?;
    Ok(TypeDef {
        id,
        description,
        fields,
    })
}

fn parse_fields(table: &Table) -> Result<Vec<Arg>, LoadError> {
    let Ok(seq): Result<Table, _> = table.get("fields") else {
        return Err(LoadError::Message(
            "type requires fields = { ... }".to_string(),
        ));
    };
    let mut fields = Vec::new();
    for pair in seq.sequence_values::<Table>() {
        let row = pair.map_err(|e| LoadError::Message(e.to_string()))?;
        fields.push(parse_arg(&row)?);
    }
    Ok(fields)
}

fn optional_types(table: &Table) -> Result<BTreeMap<String, TypeDef>, LoadError> {
    match table.get::<Value>("types") {
        Ok(Value::Nil) | Err(_) => Ok(BTreeMap::new()),
        Ok(Value::Table(t)) => {
            let mut out = BTreeMap::new();
            for pair in t.pairs::<Value, Table>() {
                let (k, inner) = pair.map_err(|e| LoadError::Message(e.to_string()))?;
                let id = match k {
                    Value::String(s) => s.to_str()?.to_string(),
                    Value::Integer(i) => {
                        optional_string(&inner, "id")?.unwrap_or_else(|| i.to_string())
                    }
                    other => {
                        return Err(LoadError::Message(format!(
                            "types keys must be strings, got {}",
                            other.type_name()
                        )));
                    }
                };
                let def = type_from_table(&inner, Some(id.clone()))?;
                out.insert(id, def);
            }
            Ok(out)
        }
        Ok(other) => Err(LoadError::Message(format!(
            "'types' must be a table, got {}",
            other.type_name()
        ))),
    }
}

fn parse_arg_from(row: &Table) -> Result<Option<ArgFrom>, LoadError> {
    let Ok(value) = row.get::<Value>("from") else {
        return Ok(None);
    };
    match value {
        Value::Nil => Ok(None),
        Value::String(s) => Ok(Some(ArgFrom::Builtin(s.to_str()?.to_string()))),
        Value::Table(t) => Ok(Some(ArgFrom::Spec(ArgFromSpec {
            template: optional_string(&t, "template")?,
            query: optional_query_string(&t)?,
            capture: optional_string(&t, "capture")?,
            extract: optional_string(&t, "extract")?,
            scope: optional_string(&t, "scope")?,
            language: optional_string(&t, "language")?,
            on_no_match: optional_string(&t, "onNoMatch")?,
            on_many_matches: optional_string(&t, "onManyMatches")?,
            join: optional_string(&t, "join")?,
        }))),
        other => Err(LoadError::Message(format!(
            "arg.from must be a string or table, got {}",
            other.type_name()
        ))),
    }
}

fn optional_query_string(table: &Table) -> Result<Option<String>, LoadError> {
    match table.get::<Value>("query") {
        Ok(Value::Nil) | Err(_) => Ok(None),
        Ok(Value::String(s)) => Ok(Some(s.to_str()?.to_string())),
        Ok(Value::Table(t)) => {
            let mut steps = Vec::new();
            for v in t.sequence_values::<String>() {
                steps.push(v.map_err(|e| LoadError::Message(e.to_string()))?);
            }
            Ok(Some(steps.join("\n---\n")))
        }
        Ok(other) => Err(LoadError::Message(format!(
            "query must be a string or list, got {}",
            other.type_name()
        ))),
    }
}

fn parse_explorer_menu(table: &Table) -> Result<Vec<ExplorerMenuEntry>, LoadError> {
    let Ok(value) = table.get::<Value>("explorerMenu") else {
        return Ok(vec![]);
    };
    match value {
        Value::Nil => Ok(vec![]),
        Value::Table(t) => {
            if t.contains_key("kind")? {
                return Ok(vec![explorer_entry(&t)?]);
            }
            let mut entries = Vec::new();
            for row in t.sequence_values::<Table>() {
                entries.push(explorer_entry(
                    &row.map_err(|e| LoadError::Message(e.to_string()))?,
                )?);
            }
            Ok(entries)
        }
        other => Err(LoadError::Message(format!(
            "explorerMenu must be a table, got {}",
            other.type_name()
        ))),
    }
}

fn explorer_entry(table: &Table) -> Result<ExplorerMenuEntry, LoadError> {
    let kind_s = required_string(table, "kind")?;
    let kind = ExplorerMenuKind::parse(&kind_s)
        .ok_or_else(|| LoadError::Message(format!("unknown explorerMenu kind '{kind_s}'")))?;
    Ok(ExplorerMenuEntry {
        kind,
        if_expr: optional_string(table, "if")?,
        args: optional_string_map_field(table, "args")?,
    })
}

fn required_string(table: &Table, key: &str) -> Result<String, LoadError> {
    table
        .get::<String>(key)
        .map_err(|_| LoadError::Message(format!("missing or invalid '{key}'")))
}

fn optional_string(table: &Table, key: &str) -> Result<Option<String>, LoadError> {
    match table.get::<Value>(key) {
        Ok(Value::Nil) | Err(_) => Ok(None),
        Ok(Value::String(s)) => Ok(Some(s.to_str()?.to_string())),
        Ok(other) => Err(LoadError::Message(format!(
            "'{key}' must be a string, got {}",
            other.type_name()
        ))),
    }
}

fn optional_bool(table: &Table, key: &str) -> Result<Option<bool>, LoadError> {
    match table.get::<Value>(key) {
        Ok(Value::Nil) | Err(_) => Ok(None),
        Ok(Value::Boolean(b)) => Ok(Some(b)),
        Ok(other) => Err(LoadError::Message(format!(
            "'{key}' must be a boolean, got {}",
            other.type_name()
        ))),
    }
}

fn optional_string_list(table: &Table, key: &str) -> Result<Vec<String>, LoadError> {
    match table.get::<Value>(key) {
        Ok(Value::Nil) | Err(_) => Ok(vec![]),
        Ok(Value::Table(t)) => {
            let mut out = Vec::new();
            for v in t.sequence_values::<String>() {
                out.push(v.map_err(|e| LoadError::Message(e.to_string()))?);
            }
            Ok(out)
        }
        Ok(other) => Err(LoadError::Message(format!(
            "'{key}' must be a list of strings, got {}",
            other.type_name()
        ))),
    }
}

fn optional_string_map_field(
    table: &Table,
    key: &str,
) -> Result<BTreeMap<String, String>, LoadError> {
    match table.get::<Value>(key) {
        Ok(Value::Nil) | Err(_) => Ok(BTreeMap::new()),
        Ok(Value::Table(t)) => string_map(&t),
        Ok(other) => Err(LoadError::Message(format!(
            "'{key}' must be a table, got {}",
            other.type_name()
        ))),
    }
}

fn optional_nested_maps(
    table: &Table,
    key: &str,
) -> Result<BTreeMap<String, BTreeMap<String, String>>, LoadError> {
    match table.get::<Value>(key) {
        Ok(Value::Nil) | Err(_) => Ok(BTreeMap::new()),
        Ok(Value::Table(t)) => {
            let mut out = BTreeMap::new();
            for pair in t.pairs::<String, Table>() {
                let (id, inner) = pair.map_err(|e| LoadError::Message(e.to_string()))?;
                out.insert(id, string_map(&inner)?);
            }
            Ok(out)
        }
        Ok(other) => Err(LoadError::Message(format!(
            "'{key}' must be a table of maps, got {}",
            other.type_name()
        ))),
    }
}

fn string_map(table: &Table) -> Result<BTreeMap<String, String>, LoadError> {
    let mut out = BTreeMap::new();
    for pair in table.pairs::<Value, Value>() {
        let (k, v) = pair.map_err(|e| LoadError::Message(e.to_string()))?;
        let key = value_to_string(&k)?;
        let val = value_to_string(&v)?;
        out.insert(key, val);
    }
    Ok(out)
}

fn value_to_string(value: &Value) -> Result<String, LoadError> {
    match value {
        Value::String(s) => Ok(s.to_str()?.to_string()),
        Value::Integer(i) => Ok(i.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Boolean(b) => Ok(b.to_string()),
        other => Err(LoadError::Message(format!(
            "expected string-like value, got {}",
            other.type_name()
        ))),
    }
}

fn map_lua_error(err: mlua::Error, source: &str) -> LoadError {
    let message = err.to_string();
    if let Some((line, column)) = parse_lua_location(&message, source) {
        LoadError::Located {
            message,
            line: Some(line),
            column: Some(column),
        }
    } else {
        LoadError::Message(message)
    }
}

fn parse_lua_location(message: &str, _source: &str) -> Option<(u32, u32)> {
    // Luau messages often look like `[string "name"]:12: error`
    let after = message.split(']').nth(1)?;
    let rest = after.strip_prefix(':')?;
    let line: u32 = rest.split(':').next()?.parse().ok()?;
    Some((line, 1))
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn load_asset_recipe() {
        let src = r#"
return {
  id = "demo.insert",
  apply = function(c) end,
}
"#;
        let asset = load_asset(src, "demo.luau").unwrap();
        assert!(matches!(asset, Asset::Recipe(r) if r.id == "demo.insert"));
    }

    #[test]
    fn load_asset_map() {
        let src = r#"return { id = "m", map = { a = "b" } }"#;
        let asset = load_asset(src, "m.luau").unwrap();
        assert!(matches!(asset, Asset::Map(m) if m.id == "m" && m.map["a"] == "b"));
    }

    #[test]
    fn load_asset_unknown() {
        let src = r#"return { id = "x" }"#;
        let err = load_asset(src, "x.luau").unwrap_err().to_string();
        assert!(err.contains("unknown asset shape"));
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
            load_asset(stack, "stack.luau").unwrap(),
            Asset::Stack(s) if s.id == "stack"
        ));
        let loaded = load_asset(component, "a.b.luau").unwrap();
        assert!(matches!(loaded, Asset::Component(c) if c.id == "a.b"));
    }
}
