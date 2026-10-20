use mlua::{Table, Value};
use std::collections::BTreeMap;

use crate::input_kind::InputKind;
use crate::model::{
    Arg, ArgFrom, ArgFromSpec, ExplorerMenuEntry, ExplorerMenuKind, OperationDef, TypeDef,
};

use super::error::LoadError;
use super::table_util::{
    optional_bool, optional_string, optional_string_list, optional_string_map_field, required_string,
};

pub(crate) fn parse_arg_from(row: &Table) -> Result<Option<ArgFrom>, LoadError> {
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

pub(crate) fn optional_query_string(table: &Table) -> Result<Option<String>, LoadError> {
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

pub(crate) fn parse_explorer_menu(table: &Table) -> Result<Vec<ExplorerMenuEntry>, LoadError> {
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

pub(crate) fn explorer_entry(table: &Table) -> Result<ExplorerMenuEntry, LoadError> {
    let kind_s = required_string(table, "kind")?;
    let kind = ExplorerMenuKind::parse(&kind_s)
        .ok_or_else(|| LoadError::Message(format!("unknown explorerMenu kind '{kind_s}'")))?;
    Ok(ExplorerMenuEntry {
        kind,
        if_expr: optional_string(table, "if")?,
        args: optional_string_map_field(table, "args")?,
    })
}

