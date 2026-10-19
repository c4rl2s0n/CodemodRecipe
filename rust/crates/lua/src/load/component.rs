use mlua::{Table, Value};

use crate::model::{Arg, ComponentDef};

use super::error::LoadError;
use super::shared::{parse_args, parse_operations};
use super::table_util::{optional_string, required_string};

pub fn load_component_from_table(table: &Table, source: &str) -> Result<ComponentDef, LoadError> {
    let id = required_string(table, "id")?;
    if id.trim().is_empty() {
        return Err(LoadError::Message("component id is required".to_string()));
    }
    let description = optional_string(table, "description")?;
    require_function(table, "parse")?;
    let has_identify = has_function(table, "identify");
    let has_verify = has_function(table, "verify");
    let has_relate = has_function(table, "relate");
    let has_to_summary = has_function(table, "toSummary");
    let operations = parse_operations(table)?;
    let (has_scaffold, scaffold_args) = parse_scaffold(table)?;
    Ok(ComponentDef {
        id,
        description,
        operations,
        has_identify,
        has_verify,
        has_relate,
        has_to_summary,
        has_scaffold,
        scaffold_args,
        source: source.to_string(),
        path: None,
    })
}

pub(crate) fn parse_scaffold(table: &Table) -> Result<(bool, Vec<Arg>), LoadError> {
    let Ok(scaffold): Result<Table, _> = table.get("scaffold") else {
        return Ok((false, Vec::new()));
    };
    match scaffold.get::<Value>("apply") {
        Ok(Value::Function(_)) => {}
        Ok(other) => {
            return Err(LoadError::Message(format!(
                "scaffold requires apply function, got {}",
                other.type_name()
            )));
        }
        Err(e) => {
            return Err(LoadError::Message(format!(
                "scaffold requires apply function: {e}"
            )));
        }
    }
    let args = parse_args(&scaffold)?;
    Ok((true, args))
}

pub(crate) fn require_function(table: &Table, key: &str) -> Result<(), LoadError> {
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

pub(crate) fn has_function(table: &Table, key: &str) -> bool {
    matches!(table.get::<Value>(key), Ok(Value::Function(_)))
}

