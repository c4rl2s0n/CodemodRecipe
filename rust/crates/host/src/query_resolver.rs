//! Resolve query list items and apply Jinja to query text (including `.scm` files).

use std::collections::BTreeMap;
use std::path::Path;

use codemod_recipe_engine::query::resolve_query_source;
use codemod_recipe_yaml::model::{QueryDefinition, QuerySpec, Recipe};

use crate::registry::RecipeRegistry;
use crate::template::render_template;

/// Expand a [`QuerySpec`] to rendered tree-sitter query step strings.
pub fn render_query_spec(
    spec: &QuerySpec,
    recipe: &Recipe,
    registry: &RecipeRegistry,
    recipe_file: Option<&Path>,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<QuerySpec, String> {
    let raw_steps: Vec<String> = match spec {
        QuerySpec::Single(s) => vec![s.clone()],
        QuerySpec::Chain(v) => v.clone(),
    };
    let mut out = Vec::with_capacity(raw_steps.len());
    for item in raw_steps {
        let body = resolve_list_item_to_body(&item, recipe, registry)?;
        let text = load_and_render_query_body(
            &body,
            recipe_file,
            registry.codemod_root(),
            args,
            maps,
            vars,
        )?;
        out.push(text);
    }
    Ok(if out.len() == 1 {
        QuerySpec::Single(out.into_iter().next().unwrap())
    } else {
        QuerySpec::Chain(out)
    })
}

fn resolve_list_item_to_body(
    item: &str,
    recipe: &Recipe,
    registry: &RecipeRegistry,
) -> Result<String, String> {
    let trimmed = item.trim();
    if trimmed.is_empty() {
        return Err("query list item must not be empty".to_string());
    }

    if let Some((lib_id, key)) = split_query_ref(trimmed) {
        if let Some(entries) = registry.queries_by_id().get(lib_id) {
            if let Some(def) = entries.get(key) {
                return Ok(def.query.clone());
            }
            return Err(format!("unknown query '{key}' in library '{lib_id}'"));
        }
    }

    if let Some(def) = recipe.queries.get(trimmed) {
        return Ok(def.query.clone());
    }

    Ok(trimmed.to_string())
}

/// `dart_queries.class_named` → (`dart_queries`, `class_named`)
fn split_query_ref(s: &str) -> Option<(&str, &str)> {
    let (lib, key) = s.split_once('.')?;
    if lib.is_empty() || key.is_empty() || key.contains('.') {
        return None;
    }
    Some((lib, key))
}

fn load_and_render_query_body(
    body: &str,
    recipe_file: Option<&Path>,
    codemod_root: &Path,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<String, String> {
    let loaded = if looks_like_query_file_path(body) {
        resolve_query_source(body, recipe_file, codemod_root)
            .map_err(|e| e.to_string())?
    } else {
        body.to_string()
    };
    render_template(&loaded, args, maps, vars)
}

fn looks_like_query_file_path(query: &str) -> bool {
    let trimmed = query.trim();
    if trimmed.contains('(') {
        return false;
    }
    trimmed.ends_with(".scm")
        || trimmed.contains('/')
        || trimmed.contains('\\')
        || (trimmed.ends_with(".yaml") && !trimmed.contains('('))
}

/// Parse a query library YAML document (`id` + `queries` map).
pub fn parse_query_library(
    root: &serde_yaml::Mapping,
) -> Result<(String, BTreeMap<String, QueryDefinition>), String> {
    let id = root
        .get(serde_yaml::Value::String("id".into()))
        .and_then(|v| v.as_str())
        .ok_or_else(|| "query library requires string field 'id'".to_string())?
        .trim()
        .to_string();
    if id.is_empty() {
        return Err("query library id must not be empty".to_string());
    }
    let queries_val = root
        .get(serde_yaml::Value::String("queries".into()))
        .ok_or_else(|| "query library requires 'queries' map".to_string())?;
    let queries_map = queries_val
        .as_mapping()
        .ok_or_else(|| "query library 'queries' must be a mapping".to_string())?;

    let mut entries = BTreeMap::new();
    for (k, v) in queries_map {
        let key = k
            .as_str()
            .ok_or_else(|| "query library keys must be strings".to_string())?
            .to_string();
        let def: QueryDefinition = serde_yaml::from_value(v.clone())
            .map_err(|e| format!("invalid query definition '{key}': {e}"))?;
        if def.query.trim().is_empty() {
            return Err(format!("query library entry '{key}' must not be empty"));
        }
        entries.insert(key, def);
    }
    Ok((id, entries))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_dotted_query_ref() {
        assert_eq!(
            split_query_ref("dart_queries.class_named"),
            Some(("dart_queries", "class_named"))
        );
        assert_eq!(split_query_ref("no_dot"), None);
    }
}
