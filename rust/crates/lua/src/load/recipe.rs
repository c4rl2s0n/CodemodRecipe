use mlua::Table;

use crate::model::Recipe;

use super::error::LoadError;
use super::shared::{
    optional_types, parse_args, parse_explorer_menu,
};
use super::table_util::{
    optional_nested_maps, optional_string, optional_string_list, optional_string_map_field,
    required_string,
};
use super::{eval_return_table, new_sandbox};

pub fn load_recipe(source: &str, chunk_name: &str) -> Result<Recipe, LoadError> {
    let lua = new_sandbox()?;
    let table = eval_return_table(&lua, source, chunk_name)?;
    load_recipe_from_table(&table, source)
}

pub fn load_recipe_from_table(table: &Table, source: &str) -> Result<Recipe, LoadError> {
    let mut recipe = recipe_from_table(table)?;
    recipe.source = source.to_string();
    if recipe.id.trim().is_empty() {
        return Err(LoadError::Message("recipe id is required".to_string()));
    }
    table
        .get::<Option<mlua::Function>>("apply")
        .map_err(|e| LoadError::Message(e.to_string()))?
        .ok_or_else(|| LoadError::Message("recipe apply function is required".to_string()))?;
    Ok(recipe)
}

fn recipe_from_table(table: &Table) -> Result<Recipe, LoadError> {
    let id = required_string(table, "id")?;
    let name = optional_string(table, "name")?;
    let description = optional_string(table, "description")?;
    let args = parse_args(table)?;
    let maps = optional_nested_maps(table, "maps")?;
    let types = optional_types(table)?;
    let queries = optional_string_map_field(table, "queries")?;
    let post_execution = optional_string_list(table, "postExecution")?;
    let explorer_menu = parse_explorer_menu(table)?;
    Ok(Recipe {
        id,
        name,
        description,
        args,
        maps,
        types,
        queries,
        post_execution,
        explorer_menu,
        source: String::new(),
        path: None,
    })
}

