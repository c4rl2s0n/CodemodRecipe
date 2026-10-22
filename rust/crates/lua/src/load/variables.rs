use mlua::Table;

use crate::model::VariablesAsset;

use super::error::LoadError;
use super::table_util::{optional_string, required_string, string_map};
use super::{eval_return_table, new_sandbox};

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

