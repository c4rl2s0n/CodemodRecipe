use codemod_recipe_yaml::model::Recipe;
use std::collections::BTreeMap;

/// Overlay recipe `defaultsTo` values for keys missing from caller args.
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

/// Returns missing required arg names after applying defaults.
pub fn missing_required_args(
    recipe: &Recipe,
    args: &BTreeMap<String, String>,
) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use codemod_recipe_yaml::model::Arg;

    fn arg(name: &str, required: bool, defaults_to: Option<&str>) -> Arg {
        Arg {
            name: name.to_string(),
            required,
            input_kind: None,
            abbr: None,
            help: None,
            defaults_to: defaults_to.map(String::from),
            options: vec![],
            allow_custom_value: None,
            context_key: None,
        }
    }

    #[test]
    fn defaults_to_satisfies_required() {
        let recipe = Recipe {
            id: "r".to_string(),
            name: None,
            description: None,
            args: vec![arg("flag", true, Some("false"))],
            maps: BTreeMap::new(),
        queries: BTreeMap::new(),
            steps: vec![],
            post_execution: vec![],
        };
        let effective = validate_required_args(&recipe, &BTreeMap::new()).unwrap();
        assert_eq!(effective.get("flag"), Some(&"false".to_string()));
    }

    #[test]
    fn caller_overrides_default() {
        let recipe = Recipe {
            id: "r".to_string(),
            name: None,
            description: None,
            args: vec![arg("flag", false, Some("false"))],
            maps: BTreeMap::new(),
        queries: BTreeMap::new(),
            steps: vec![],
            post_execution: vec![],
        };
        let mut caller = BTreeMap::new();
        caller.insert("flag".to_string(), "true".to_string());
        let effective = resolve_effective_args(&recipe, &caller);
        assert_eq!(effective.get("flag"), Some(&"true".to_string()));
    }

    #[test]
    fn child_defaults_visible_after_orchestrator_expand() {
        use codemod_recipe_yaml::compose::expand_recipe_references;
        use codemod_recipe_yaml::model::{Recipe, Step};
        use std::collections::BTreeMap;

        let child = Recipe {
            id: "child".to_string(),
            name: None,
            description: None,
            args: vec![arg("includeRepo", true, Some("false"))],
            maps: BTreeMap::new(),
        queries: BTreeMap::new(),
            steps: vec![],
            post_execution: vec![],
        };
        let mut registry = BTreeMap::new();
        registry.insert("child".to_string(), child);
        let parent = Recipe {
            id: "parent".to_string(),
            name: None,
            description: None,
            args: vec![],
            maps: BTreeMap::new(),
        queries: BTreeMap::new(),
            steps: vec![Step::RecipeRef(codemod_recipe_yaml::model::RecipeRef {
                id: "child".to_string(),
                with: BTreeMap::new(),
            })],
            post_execution: vec![],
        };
        let expanded = expand_recipe_references(&parent, &registry).unwrap();
        let effective = validate_required_args(&expanded, &BTreeMap::new()).unwrap();
        assert_eq!(effective.get("includeRepo"), Some(&"false".to_string()));
    }
}
