use mlua::{Table, Value};
use std::collections::BTreeMap;

use crate::input_kind::InputKind;
use crate::model::{Arg, TypeDef};

use super::error::LoadError;
use super::ops::parse_arg;
use super::table_util::{optional_string, required_string};

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

