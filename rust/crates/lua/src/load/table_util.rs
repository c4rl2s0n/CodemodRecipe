use mlua::{Table, Value};
use std::collections::BTreeMap;

use super::error::LoadError;

pub(crate) fn string_array(table: &Table) -> Result<Vec<String>, LoadError> {
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

pub(crate) fn required_string(table: &Table, key: &str) -> Result<String, LoadError> {
    table
        .get::<String>(key)
        .map_err(|_| LoadError::Message(format!("missing or invalid '{key}'")))
}

pub(crate) fn optional_string(table: &Table, key: &str) -> Result<Option<String>, LoadError> {
    match table.get::<Value>(key) {
        Ok(Value::Nil) | Err(_) => Ok(None),
        Ok(Value::String(s)) => Ok(Some(s.to_str()?.to_string())),
        Ok(other) => Err(LoadError::Message(format!(
            "'{key}' must be a string, got {}",
            other.type_name()
        ))),
    }
}

pub(crate) fn optional_bool(table: &Table, key: &str) -> Result<Option<bool>, LoadError> {
    match table.get::<Value>(key) {
        Ok(Value::Nil) | Err(_) => Ok(None),
        Ok(Value::Boolean(b)) => Ok(Some(b)),
        Ok(other) => Err(LoadError::Message(format!(
            "'{key}' must be a boolean, got {}",
            other.type_name()
        ))),
    }
}

pub(crate) fn optional_string_list(table: &Table, key: &str) -> Result<Vec<String>, LoadError> {
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

pub(crate) fn optional_string_map_field(
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

pub(crate) fn optional_nested_maps(
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

pub(crate) fn string_map(table: &Table) -> Result<BTreeMap<String, String>, LoadError> {
    let mut out = BTreeMap::new();
    for pair in table.pairs::<Value, Value>() {
        let (k, v) = pair.map_err(|e| LoadError::Message(e.to_string()))?;
        let key = value_to_string(&k)?;
        let val = value_to_string(&v)?;
        out.insert(key, val);
    }
    Ok(out)
}

pub(crate) fn value_to_string(value: &Value) -> Result<String, LoadError> {
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

pub(crate) fn map_lua_error(err: mlua::Error, source: &str) -> LoadError {
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

pub(crate) fn parse_lua_location(message: &str, _source: &str) -> Option<(u32, u32)> {
    // Luau messages often look like `[string "name"]:12: error`
    let after = message.split(']').nth(1)?;
    let rest = after.strip_prefix(':')?;
    let line: u32 = rest.split(':').next()?.parse().ok()?;
    Some((line, 1))
}

