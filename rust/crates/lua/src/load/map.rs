use mlua::Table;

use crate::model::MapAsset;

use super::error::LoadError;
use super::table_util::{optional_string, required_string, string_map};
use super::{eval_return_table, new_sandbox};

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

