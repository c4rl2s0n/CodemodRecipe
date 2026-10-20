use mlua::{Table, Value};
use std::collections::BTreeMap;

use crate::input_kind::InputKind;
use crate::model::{
    Arg, ArgFrom, ArgFromSpec, ExplorerMenuEntry, ExplorerMenuKind, OperationDef, TypeDef,
};

use super::error::LoadError;
use super::explorer::parse_arg_from;
use super::table_util::{
    optional_bool, optional_string, optional_string_list, optional_string_map_field, required_string,
};

pub(crate) fn parse_operations(table: &Table) -> Result<Vec<OperationDef>, LoadError> {
    let Ok(ops): Result<Table, _> = table.get("operations") else {
        return Ok(vec![]);
    };
    let mut out = Vec::new();
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
        let description = optional_string(&op_tbl, "description")?;
        let args = parse_args(&op_tbl)?;
        out.push(OperationDef {
            name,
            description,
            args,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

pub(crate) fn parse_args(table: &Table) -> Result<Vec<Arg>, LoadError> {
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

pub(crate) fn parse_arg(row: &Table) -> Result<Arg, LoadError> {
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

pub(crate) fn parse_input_kind(row: &Table) -> Result<InputKind, LoadError> {
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

