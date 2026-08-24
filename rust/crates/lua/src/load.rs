use mlua::{Lua, Table, Value};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use thiserror::Error;

use crate::model::{
    Arg, ArgFrom, ArgFromSpec, ExplorerMenuEntry, ExplorerMenuKind, MapAsset, QueryLibAsset,
    Recipe, VariablesAsset,
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

pub fn load_recipe(source: &str, chunk_name: &str) -> Result<Recipe, LoadError> {
    let lua = new_sandbox()?;
    let table = eval_return_table(&lua, source, chunk_name)?;
    let mut recipe = recipe_from_table(&table)?;
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
    let id: String = required_string(&table, "id")?;
    let description: Option<String> = optional_string(&table, "description")?;
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
    let id: String = required_string(&table, "id")?;
    let description: Option<String> = optional_string(&table, "description")?;
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
    let id: String = required_string(&table, "id")?;
    let queries_tbl: Table = table
        .get("queries")
        .map_err(|_| LoadError::Message("query lib requires queries = { ... }".to_string()))?;
    Ok(QueryLibAsset {
        id,
        queries: string_map(&queries_tbl)?,
    })
}

fn recipe_from_table(table: &Table) -> Result<Recipe, LoadError> {
    let id = required_string(table, "id")?;
    let name = optional_string(table, "name")?;
    let description = optional_string(table, "description")?;
    let args = parse_args(table)?;
    let maps = optional_nested_maps(table, "maps")?;
    let queries = optional_string_map_field(table, "queries")?;
    let post_execution = optional_string_list(table, "postExecution")?;
    let explorer_menu = parse_explorer_menu(table)?;
    Ok(Recipe {
        id,
        name,
        description,
        args,
        maps,
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
        args.push(Arg {
            name: required_string(&row, "name")?,
            required: optional_bool(&row, "required")?.unwrap_or(false),
            input_kind: optional_string(&row, "inputKind")?,
            abbr: optional_string(&row, "abbr")?,
            help: optional_string(&row, "help")?,
            defaults_to: optional_string(&row, "defaultsTo")?,
            options: optional_string_list(&row, "options")?,
            allow_custom_value: optional_bool(&row, "allowCustomValue")?,
            from: parse_arg_from(&row)?,
        });
    }
    Ok(args)
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
    }

    #[test]
    fn rejects_missing_apply() {
        let src = r#"return { id = "x" }"#;
        assert!(load_recipe(src, "x.luau").is_err());
    }
}
