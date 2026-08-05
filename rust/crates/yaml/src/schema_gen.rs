//! Emit JSON Schema documents from [`crate::model`] via schemars + [`crate::dsl_vocabulary`].

use crate::description_for_key;
use crate::dsl;
use crate::dsl_vocabulary::{all_entries, VocabKind};
use crate::model::{MapAsset, Recipe, VariablesAsset};
use schemars::gen::SchemaSettings;
use schemars::schema::RootSchema;
use serde_json::{Map, Value};

fn draft07_root_schema_for<T: schemars::JsonSchema>() -> RootSchema {
    let settings = SchemaSettings::draft07().with(|s| {
        s.option_nullable = false;
        s.option_add_null_type = false;
    });
    settings.into_generator().into_root_schema_for::<T>()
}

fn root_to_value(root: RootSchema) -> Value {
    serde_json::to_value(root).expect("schemars RootSchema serializes")
}

/// Walk schema objects and attach ENTRIES prose to properties / discriminator keys.
fn merge_vocab_descriptions(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::Object(props)) = map.get_mut("properties") {
                let keys: Vec<String> = props.keys().cloned().collect();
                for key in keys {
                    if let Some(prop) = props.get_mut(&key) {
                        apply_description(prop, &key);
                        merge_vocab_descriptions(prop);
                    }
                }
            }
            if map.contains_key("definitions") {
                if let Some(defs) = map.get_mut("definitions") {
                    merge_vocab_descriptions(defs);
                }
            } else if let Some(defs) = map.get_mut("$defs") {
                merge_vocab_descriptions(defs);
            }
            for (k, v) in map.iter_mut() {
                if k != "properties" && k != "definitions" && k != "$defs" {
                    merge_vocab_descriptions(v);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                merge_vocab_descriptions(item);
            }
        }
        _ => {}
    }
}

fn apply_description(prop: &mut Value, wire: &str) {
    let Value::Object(obj) = prop else {
        return;
    };
    if let Some(entry) = all_entries()
        .iter()
        .find(|e| e.wire == wire && matches!(e.kind, VocabKind::StepKind | VocabKind::OpKind))
    {
        obj.insert(
            "description".into(),
            Value::String(entry.description.to_string()),
        );
        return;
    }
    if obj.contains_key("description") {
        return;
    }
    if let Some(d) = description_for_key(wire) {
        obj.insert("description".into(), Value::String(d.to_string()));
    }
}

/// Prefer draft-07 `definitions` key (Red Hat YAML / existing artifacts).
fn normalize_definitions_key(schema: &mut Value) {
    let Some(obj) = schema.as_object_mut() else {
        return;
    };
    if let Some(defs) = obj.remove("$defs") {
        obj.insert("definitions".into(), defs);
    }
}

/// Open object shapes for forward-compatible YAML (match prior schema leniency).
fn open_object_additional_properties(schema: &mut Value) {
    match schema {
        Value::Object(map) => {
            let is_object = map
                .get("type")
                .and_then(|t| t.as_str())
                .is_some_and(|t| t == "object")
                || map.contains_key("properties");
            let has_max_props = map
                .get("maxProperties")
                .and_then(|v| v.as_u64())
                .is_some_and(|n| n == 1);
            if is_object && map.contains_key("properties") && !has_max_props {
                map.entry("additionalProperties".to_string())
                    .or_insert(Value::Bool(true));
            }
            for (k, v) in map.iter_mut() {
                if k != "additionalProperties" {
                    open_object_additional_properties(v);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                open_object_additional_properties(item);
            }
        }
        _ => {}
    }
}

/// Arg entries reject unknown keys (match `#[serde(deny_unknown_fields)]` on [`Arg`]).
fn close_arg_additional_properties(schema: &mut Value) {
    let Some(defs) = schema
        .as_object_mut()
        .and_then(|o| o.get_mut("definitions"))
        .and_then(|d| d.as_object_mut())
    else {
        return;
    };
    let Some(arg) = defs.get_mut("arg").and_then(|a| a.as_object_mut()) else {
        return;
    };
    arg.insert("additionalProperties".into(), Value::Bool(false));
}

fn finalize_document(
    mut schema: Value,
    id: &str,
    title: &str,
    required: &[&str],
) -> Value {
    normalize_definitions_key(&mut schema);
    merge_vocab_descriptions(&mut schema);
    open_object_additional_properties(&mut schema);
    close_arg_additional_properties(&mut schema);

    let Some(obj) = schema.as_object_mut() else {
        return schema;
    };
    obj.insert(
        "$schema".into(),
        Value::String("http://json-schema.org/draft-07/schema#".into()),
    );
    obj.insert("$id".into(), Value::String(id.into()));
    obj.insert("title".into(), Value::String(title.into()));
    if !required.is_empty() {
        obj.insert(
            "required".into(),
            Value::Array(required.iter().map(|s| Value::String((*s).into())).collect()),
        );
    }
    // Drop schemars meta noise that editors do not need.
    obj.remove("format");
    schema
}

/// Build `recipe.schema.json` from [`Recipe`] and nested model types.
pub fn recipe_schema() -> Value {
    let root = root_to_value(draft07_root_schema_for::<Recipe>());
    finalize_document(
        root,
        "https://codemod-recipe.dev/schemas/recipe.schema.json",
        "Codemod Recipe",
        &[dsl::recipe::field::ID, dsl::recipe::field::STEPS],
    )
}

pub fn map_schema() -> Value {
    let root = root_to_value(draft07_root_schema_for::<MapAsset>());
    finalize_document(
        root,
        "https://codemod-recipe.dev/schemas/map.schema.json",
        "Codemod Recipe Map",
        &[dsl::map_asset::field::ID, dsl::map_asset::field::MAP],
    )
}

pub fn variables_schema() -> Value {
    let root = root_to_value(draft07_root_schema_for::<VariablesAsset>());
    finalize_document(
        root,
        "https://codemod-recipe.dev/schemas/variables.schema.json",
        "Codemod Recipe Variables",
        &[
            dsl::variables_asset::field::ID,
            dsl::variables_asset::field::VALUES,
        ],
    )
}

/// Resolve a `$ref` like `#/definitions/editStep` against a schema document.
pub fn resolve_definition<'a>(schema: &'a Value, name: &str) -> Option<&'a Value> {
    schema
        .pointer(&format!("/definitions/{name}"))
        .or_else(|| schema.pointer(&format!("/$defs/{name}")))
}

/// Property keys of an object schema (empty if not an object with properties).
pub fn object_property_keys(schema: &Value) -> Vec<String> {
    schema
        .get("properties")
        .and_then(|p| p.as_object())
        .map(|o| o.keys().cloned().collect())
        .unwrap_or_default()
}

/// Follow `$ref` once when present.
pub fn deref_schema<'a>(root: &'a Value, schema: &'a Value) -> &'a Value {
    if let Some(r) = schema.get("$ref").and_then(|v| v.as_str()) {
        if let Some(name) = r.strip_prefix("#/definitions/") {
            if let Some(d) = resolve_definition(root, name) {
                return d;
            }
        }
        if let Some(name) = r.strip_prefix("#/$defs/") {
            if let Some(d) = resolve_definition(root, name) {
                return d;
            }
        }
    }
    schema
}

/// Collect string enums appearing under object properties, keyed for editor surface.
pub fn collect_field_enums(root: &Value) -> Map<String, Value> {
    let mut out = Map::new();
    let mut visiting_defs = std::collections::HashSet::new();
    visit_enums(root, root, &mut visiting_defs, &mut out);
    out
}

fn visit_enums(
    root: &Value,
    node: &Value,
    visiting_defs: &mut std::collections::HashSet<String>,
    out: &mut Map<String, Value>,
) {
    if let Some(r) = node.get("$ref").and_then(|v| v.as_str()) {
        let name = r
            .strip_prefix("#/definitions/")
            .or_else(|| r.strip_prefix("#/$defs/"));
        if let Some(name) = name {
            if !visiting_defs.insert(name.to_string()) {
                return;
            }
            if let Some(def) = resolve_definition(root, name) {
                visit_enums(root, def, visiting_defs, out);
            }
            visiting_defs.remove(name);
        }
        return;
    }

    if let Some(props) = node.get("properties").and_then(|p| p.as_object()) {
        for (key, prop) in props {
            if let Some(values) = enum_strings(prop) {
                let enum_id = surface_enum_id(key);
                out.insert(
                    enum_id,
                    Value::Array(values.into_iter().map(Value::String).collect()),
                );
            } else if let Some(r) = prop.get("$ref").and_then(|v| v.as_str()) {
                // Enum defined as a $ref'd type (e.g. inputKind, anchor).
                let name = r
                    .strip_prefix("#/definitions/")
                    .or_else(|| r.strip_prefix("#/$defs/"));
                if let Some(name) = name {
                    if let Some(def) = resolve_definition(root, name) {
                        if let Some(values) = enum_strings(def) {
                            let enum_id = surface_enum_id(key);
                            out.insert(
                                enum_id,
                                Value::Array(values.into_iter().map(Value::String).collect()),
                            );
                        }
                    }
                }
            }
            visit_enums(root, prop, visiting_defs, out);
        }
    }
    if let Some(defs) = node
        .get("definitions")
        .or_else(|| node.get("$defs"))
        .and_then(|d| d.as_object())
    {
        for (name, def) in defs {
            if let Some(values) = enum_strings(def) {
                let enum_id = surface_enum_id(name);
                out.entry(enum_id).or_insert_with(|| {
                    Value::Array(values.into_iter().map(Value::String).collect())
                });
            }
            if visiting_defs.insert(name.clone()) {
                visit_enums(root, def, visiting_defs, out);
                visiting_defs.remove(name);
            }
        }
    }
    if let Some(one_of) = node.get("oneOf").and_then(|v| v.as_array()) {
        for alt in one_of {
            visit_enums(root, alt, visiting_defs, out);
        }
    }
    if let Some(items) = node.get("items") {
        visit_enums(root, items, visiting_defs, out);
    }
}

fn enum_strings(schema: &Value) -> Option<Vec<String>> {
    let arr = schema.get("enum")?.as_array()?;
    let mut out = Vec::new();
    for v in arr {
        out.push(v.as_str()?.to_string());
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn surface_enum_id(field_or_def: &str) -> String {
    match field_or_def {
        "kind" | "ExplorerMenuKind" => "explorerMenuKind".to_string(),
        "InsertAnchor" => "anchor".to_string(),
        "IfExistsStrategy" => "ifExists".to_string(),
        "IfMissingStrategy" => "ifMissing".to_string(),
        "ArgInputKind" => "inputKind".to_string(),
        "LetExtract" => "extract".to_string(),
        "LetOnNoMatch" => "onNoMatch".to_string(),
        "LetOnManyMatches" => "onManyMatches".to_string(),
        "anchor" => "anchor".to_string(),
        other => other.to_string(),
    }
}

/// Object branch of a `oneOf` that has `properties` (e.g. recipeRef).
pub fn object_branch_of_one_of<'a>(root: &'a Value, schema: &'a Value) -> Option<&'a Value> {
    let schema = deref_schema(root, schema);
    if schema.get("properties").is_some() {
        return Some(schema);
    }
    let one_of = schema.get("oneOf")?.as_array()?;
    one_of.iter().find_map(|alt| {
        let alt = deref_schema(root, alt);
        if alt.get("properties").is_some() {
            Some(alt)
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recipe_schema_from_model_has_edit_fields() {
        let schema = recipe_schema();
        assert!(schema["required"]
            .as_array()
            .unwrap()
            .iter()
            .any(|v| v == "id"));
        let edit = resolve_definition(&schema, "editStep").expect("editStep");
        let keys = object_property_keys(edit);
        assert!(keys.iter().any(|k| k == "path"));
        assert!(keys.iter().any(|k| k == "ops"));
        assert!(keys.iter().any(|k| k == "whenNot"));
    }

    #[test]
    fn input_kind_enum_present() {
        let schema = recipe_schema();
        let enums = collect_field_enums(&schema);
        let values = enums.get("inputKind").expect("inputKind");
        assert!(values.as_array().unwrap().iter().any(|v| v == "file"));
    }
}
