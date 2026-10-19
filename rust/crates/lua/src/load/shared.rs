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

pub(crate) fn type_from_table(table: &Table, id_override: Option<String>) -> Result<TypeDef, LoadError> {
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

pub(crate) fn parse_fields(table: &Table) -> Result<Vec<Arg>, LoadError> {
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

pub(crate) fn optional_types(table: &Table) -> Result<BTreeMap<String, TypeDef>, LoadError> {
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

