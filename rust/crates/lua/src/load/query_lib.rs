use mlua::Table;

use crate::model::QueryLibAsset;

use super::error::LoadError;
use super::table_util::{required_string, string_map};
use super::{eval_return_table, new_sandbox};

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

