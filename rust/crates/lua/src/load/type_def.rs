use mlua::Table;

use crate::model::TypeDef;

use super::error::LoadError;
use super::shared::type_from_table;
use super::{eval_return_table, new_sandbox};

pub fn load_type(source: &str, chunk_name: &str) -> Result<TypeDef, LoadError> {
    let lua = new_sandbox()?;
    let table = eval_return_table(&lua, source, chunk_name)?;
    load_type_from_table(&table)
}

pub fn load_type_from_table(table: &Table) -> Result<TypeDef, LoadError> {
    type_from_table(table, None)
}

