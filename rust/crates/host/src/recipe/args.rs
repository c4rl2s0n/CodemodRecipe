use std::collections::BTreeMap;

use codemod_recipe_lua::model::{Arg, Recipe, TypeDef};
use codemod_recipe_lua::InputKind;
use serde_json::{json, Value};

pub type ArgMap = BTreeMap<String, Value>;

pub fn strings_to_arg_map(map: &BTreeMap<String, String>) -> ArgMap {
    map.iter()
        .map(|(k, v)| (k.clone(), Value::String(v.clone())))
        .collect()
}

pub fn resolve_effective_args(recipe: &Recipe, caller: &ArgMap) -> ArgMap {
    let mut effective = caller.clone();
    for arg in &recipe.args {
        if effective.contains_key(&arg.name) {
            continue;
        }
        if let Some(default) = default_value(arg) {
            effective.insert(arg.name.clone(), default);
        }
    }
    effective
}

fn default_value(arg: &Arg) -> Option<Value> {
    if let Some(raw) = &arg.defaults_to {
        return Some(parse_default(arg, raw));
    }
    match &arg.input_kind {
        InputKind::List(_) => Some(json!([])),
        InputKind::Bool => Some(Value::Bool(false)),
        _ => None,
    }
}

fn parse_default(arg: &Arg, raw: &str) -> Value {
    match arg.input_kind {
        InputKind::Bool => Value::Bool(raw == "true"),
        _ => Value::String(raw.to_string()),
    }
}

pub fn missing_required_args(recipe: &Recipe, args: &ArgMap) -> Vec<String> {
    recipe
        .args
        .iter()
        .filter(|arg| arg.required && is_missing(&arg.input_kind, args.get(&arg.name)))
        .map(|arg| arg.name.clone())
        .collect()
}

fn is_missing(kind: &InputKind, value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => true,
        Some(Value::String(s)) if s.is_empty() && !kind.is_list() => true,
        _ => false,
    }
}

pub fn validate_required_args(recipe: &Recipe, caller: &ArgMap) -> Result<ArgMap, String> {
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

pub fn validate_typed_args(
    recipe: &Recipe,
    types: &BTreeMap<String, TypeDef>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    args: &ArgMap,
) -> Result<ArgMap, String> {
    let effective = validate_required_args(recipe, args)?;
    let mut merged_types = types.clone();
    for (id, def) in &recipe.types {
        merged_types
            .entry(id.clone())
            .or_insert_with(|| def.clone());
    }
    for arg in &recipe.args {
        if let Some(value) = effective.get(&arg.name) {
            validate_value(&arg.name, &arg.input_kind, arg, value, &merged_types, maps)?;
        }
    }
    Ok(effective)
}

fn validate_value(
    path: &str,
    kind: &InputKind,
    spec: &Arg,
    value: &Value,
    types: &BTreeMap<String, TypeDef>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<(), String> {
    match kind {
        InputKind::List(inner) => {
            let Value::Array(items) = value else {
                return Err(format!("{path} must be a list"));
            };
            for (i, item) in items.iter().enumerate() {
                validate_value(&format!("{path}[{i}]"), inner, spec, item, types, maps)?;
            }
            Ok(())
        }
        InputKind::Type(id) => {
            let def = types
                .get(id)
                .ok_or_else(|| format!("unknown type '{id}' for {path}"))?;
            let Value::Object(obj) = value else {
                return Err(format!("{path} must be an object of type {id}"));
            };
            for field in &def.fields {
                if field.required && is_missing(&field.input_kind, obj.get(&field.name)) {
                    return Err(format!("Missing required field {path}.{}", field.name));
                }
                if let Some(v) = obj.get(&field.name) {
                    validate_value(
                        &format!("{path}.{}", field.name),
                        &field.input_kind,
                        field,
                        v,
                        types,
                        maps,
                    )?;
                }
            }
            Ok(())
        }
        InputKind::Bool => {
            if value.as_bool().is_some()
                || matches!(value, Value::String(s) if s == "true" || s == "false")
            {
                Ok(())
            } else {
                Err(format!("{path} must be a boolean"))
            }
        }
        InputKind::Choice => {
            let Some(s) = value.as_str() else {
                return Err(format!("{path} must be a string"));
            };
            let allowed = choice_options(spec, maps);
            if allowed.is_empty() || allowed.iter().any(|o| o == s) {
                Ok(())
            } else {
                Err(format!("{path} must be one of: {}", allowed.join(", ")))
            }
        }
        InputKind::Text | InputKind::File | InputKind::Directory => {
            if value.as_str().is_some() {
                Ok(())
            } else {
                Err(format!("{path} must be a string"))
            }
        }
    }
}

pub fn choice_options(arg: &Arg, maps: &BTreeMap<String, BTreeMap<String, String>>) -> Vec<String> {
    if !arg.options.is_empty() {
        return arg.options.clone();
    }
    if let Some(map_id) = &arg.options_from_map {
        if let Some(map) = maps.get(map_id) {
            return map.keys().cloned().collect();
        }
    }
    Vec::new()
}

pub fn merged_types_for(
    recipe: &Recipe,
    global: &BTreeMap<String, TypeDef>,
) -> BTreeMap<String, TypeDef> {
    let mut out = global.clone();
    for (id, def) in &recipe.types {
        out.entry(id.clone()).or_insert_with(|| def.clone());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use codemod_recipe_lua::model::{Arg, Recipe, TypeDef};

    fn arg(name: &str, kind: InputKind, required: bool) -> Arg {
        Arg {
            name: name.to_string(),
            required,
            input_kind: kind,
            abbr: None,
            help: None,
            defaults_to: None,
            options: vec![],
            options_from_map: None,
            allow_custom_value: None,
            from: None,
        }
    }

    fn recipe(args: Vec<Arg>, types: BTreeMap<String, TypeDef>) -> Recipe {
        Recipe {
            id: "t".to_string(),
            name: None,
            description: None,
            args,
            maps: BTreeMap::new(),
            types,
            queries: BTreeMap::new(),
            post_execution: vec![],
            explorer_menu: vec![],
            source: String::new(),
            path: None,
        }
    }

    #[test]
    fn validates_list_of_named_type() {
        let mut types = BTreeMap::new();
        types.insert(
            "ClassProperty".to_string(),
            TypeDef {
                id: "ClassProperty".to_string(),
                description: None,
                fields: vec![
                    arg("type", InputKind::Text, true),
                    arg("name", InputKind::Text, true),
                    arg("initialValue", InputKind::Text, false),
                ],
            },
        );
        let recipe = recipe(
            vec![arg(
                "properties",
                InputKind::List(Box::new(InputKind::Type("ClassProperty".to_string()))),
                false,
            )],
            types.clone(),
        );
        let args = BTreeMap::from([(
            "properties".to_string(),
            json!([{ "type": "int", "name": "count" }]),
        )]);
        assert!(validate_typed_args(&recipe, &types, &BTreeMap::new(), &args).is_ok());
    }

    #[test]
    fn rejects_choice_not_in_options() {
        let mut choice = arg("kind", InputKind::Choice, true);
        choice.options = vec!["a".to_string(), "b".to_string()];
        let recipe = recipe(vec![choice], BTreeMap::new());
        let args = BTreeMap::from([("kind".to_string(), json!("z"))]);
        assert!(validate_typed_args(&recipe, &BTreeMap::new(), &BTreeMap::new(), &args).is_err());
    }
}
