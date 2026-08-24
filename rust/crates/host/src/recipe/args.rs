use std::collections::BTreeMap;

use codemod_recipe_lua::model::Recipe;

pub fn resolve_effective_args(
    recipe: &Recipe,
    caller: &BTreeMap<String, String>,
) -> BTreeMap<String, String> {
    let mut effective = caller.clone();
    for arg in &recipe.args {
        if effective.contains_key(&arg.name) {
            continue;
        }
        if let Some(default) = &arg.defaults_to {
            effective.insert(arg.name.clone(), default.clone());
        }
    }
    effective
}

pub fn missing_required_args(recipe: &Recipe, args: &BTreeMap<String, String>) -> Vec<String> {
    recipe
        .args
        .iter()
        .filter(|arg| {
            arg.required && {
                let value = args.get(&arg.name);
                value.is_none() || value.is_some_and(|v| v.is_empty())
            }
        })
        .map(|arg| arg.name.clone())
        .collect()
}

pub fn validate_required_args(
    recipe: &Recipe,
    caller: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, String> {
    let effective = resolve_effective_args(recipe, caller);
    let missing = missing_required_args(recipe, &effective);
    if !missing.is_empty() {
        return Err(format!(
            "Missing required arguments: {}",
            missing.join(", ")
        ));
    }
    Ok(effective)
}
