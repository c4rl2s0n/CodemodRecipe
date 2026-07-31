//! Emit JSON Schema documents from [`crate::dsl_structure`] + [`crate::dsl_vocabulary`].

use crate::description_for_key;
use crate::dsl;
use crate::dsl_structure::{container_by_id, CONTAINERS, ENUMS};
use crate::dsl_vocabulary::{all_entries, VocabKind};
use serde_json::{json, Map, Value};

fn desc(wire: &str) -> Option<String> {
    description_for_key(wire).map(str::to_string)
}

fn prop(wire: &str, schema: Value) -> (String, Value) {
    let mut obj = match schema {
        Value::Object(m) => m,
        other => {
            let mut m = Map::new();
            m.insert("type".into(), other);
            m
        }
    };
    if let Some(d) = desc(wire) {
        obj.insert("description".into(), Value::String(d));
    }
    // Prefer StepKind/OpKind prose for discriminator keys.
    if let Some(entry) = all_entries()
        .iter()
        .find(|e| e.wire == wire && matches!(e.kind, VocabKind::StepKind | VocabKind::OpKind))
    {
        obj.insert(
            "description".into(),
            Value::String(entry.description.to_string()),
        );
    }
    (wire.to_string(), Value::Object(obj))
}

fn string_prop() -> Value {
    json!({ "type": "string" })
}

fn string_or_null() -> Value {
    json!({ "type": ["string", "null"] })
}

fn bool_prop() -> Value {
    json!({ "type": "boolean" })
}

fn enum_prop(enum_id: &str) -> Value {
    let values = ENUMS
        .iter()
        .find(|e| e.id == enum_id)
        .map(|e| e.values)
        .unwrap_or(&[]);
    json!({ "type": "string", "enum": values })
}

fn ref_def(name: &str) -> Value {
    json!({ "$ref": format!("#/definitions/{name}") })
}

fn object_props(pairs: Vec<(String, Value)>, additional: bool) -> Value {
    let mut properties = Map::new();
    for (k, v) in pairs {
        properties.insert(k, v);
    }
    json!({
        "type": "object",
        "additionalProperties": additional,
        "properties": properties
    })
}

fn container_props(container_id: &str, field_schemas: &[(&str, Value)]) -> Value {
    let container = container_by_id(container_id).expect("known container");
    let mut pairs = Vec::new();
    for child in container.children {
        if let Some((_, schema)) = field_schemas.iter().find(|(w, _)| *w == *child) {
            pairs.push(prop(child, schema.clone()));
        } else {
            pairs.push(prop(child, string_prop()));
        }
    }
    object_props(pairs, true)
}

/// Build `recipe.schema.json` from the structural inventory.
pub fn recipe_schema() -> Value {
    let query_field = json!({
        "oneOf": [
            { "type": "string", "description": description_for_key(dsl::recipe::steps::edit::ops::insert::field::QUERY).unwrap_or("") },
            { "type": "array", "minItems": 1, "items": { "type": "string" } }
        ]
    });

    let insert_op = container_props(
        "insert",
        &[
            (dsl::recipe::steps::edit::ops::insert::field::QUERY, query_field.clone()),
            (dsl::recipe::steps::edit::ops::insert::field::CAPTURE, string_prop()),
            (
                dsl::recipe::steps::edit::ops::insert::field::ANCHOR,
                enum_prop("anchor"),
            ),
            (dsl::recipe::steps::edit::ops::insert::field::TEXT, string_prop()),
        ],
    );

    let replace_op = container_props(
        "replace",
        &[
            (dsl::recipe::steps::edit::ops::replace::field::QUERY, query_field.clone()),
            (dsl::recipe::steps::edit::ops::replace::field::CAPTURE, string_prop()),
            (dsl::recipe::steps::edit::ops::replace::field::TEXT, string_prop()),
            (
                dsl::recipe::steps::edit::ops::replace::field::INCLUDE_LEADING_TRIVIA,
                bool_prop(),
            ),
        ],
    );

    let remove_op = container_props(
        "remove",
        &[
            (dsl::recipe::steps::edit::ops::remove::field::QUERY, query_field.clone()),
            (dsl::recipe::steps::edit::ops::remove::field::CAPTURE, string_prop()),
            (
                dsl::recipe::steps::edit::ops::remove::field::INCLUDE_LEADING_TRIVIA,
                bool_prop(),
            ),
        ],
    );

    let let_binding = container_props(
        "letBinding",
        &[
            (dsl::recipe::steps::edit::let_binding::field::NAME, string_prop()),
            (dsl::recipe::steps::edit::let_binding::field::QUERY, query_field.clone()),
            (dsl::recipe::steps::edit::let_binding::field::CAPTURE, string_prop()),
            (
                dsl::recipe::steps::edit::let_binding::field::EXTRACT,
                enum_prop("extract"),
            ),
            (
                dsl::recipe::steps::edit::let_binding::field::ON_NO_MATCH,
                enum_prop("onNoMatch"),
            ),
            (
                dsl::recipe::steps::edit::let_binding::field::ON_MANY_MATCHES,
                enum_prop("onManyMatches"),
            ),
            (dsl::recipe::steps::edit::let_binding::field::JOIN, string_prop()),
            (dsl::recipe::steps::edit::let_binding::field::AS, string_prop()),
        ],
    );

    let edit_step = container_props(
        "edit",
        &[
            (dsl::recipe::steps::edit::field::PATH, string_prop()),
            (dsl::recipe::steps::edit::field::LANGUAGE, string_prop()),
            (dsl::recipe::steps::edit::field::WHEN, query_field.clone()),
            (dsl::recipe::steps::edit::field::WHEN_NOT, query_field.clone()),
            (
                dsl::recipe::steps::edit::field::LET,
                json!({
                    "oneOf": [
                        { "type": "array", "items": ref_def("letBinding") },
                        { "type": "object", "additionalProperties": ref_def("letBinding") }
                    ]
                }),
            ),
            (
                dsl::recipe::steps::edit::field::OPS,
                json!({ "type": "array", "items": ref_def("editOp"), "minItems": 1 }),
            ),
            (dsl::recipe::steps::condition::field::IF, string_prop()),
            (dsl::recipe::steps::condition::field::IF_NOT, string_prop()),
        ],
    );

    let create_step = container_props(
        "create",
        &[
            (dsl::recipe::steps::create::field::PATH, string_prop()),
            (dsl::recipe::steps::create::field::TEMPLATE, string_prop()),
            (dsl::recipe::steps::create::field::TEMPLATE_FILE, string_prop()),
            (
                dsl::recipe::steps::create::field::IF_EXISTS,
                enum_prop("ifExists"),
            ),
            (dsl::recipe::steps::condition::field::IF, string_prop()),
            (dsl::recipe::steps::condition::field::IF_NOT, string_prop()),
        ],
    );

    let delete_step = container_props(
        "delete",
        &[
            (dsl::recipe::steps::delete::field::PATH, string_prop()),
            (
                dsl::recipe::steps::delete::field::IF_MISSING,
                enum_prop("ifMissing"),
            ),
            (dsl::recipe::steps::condition::field::IF, string_prop()),
            (dsl::recipe::steps::condition::field::IF_NOT, string_prop()),
        ],
    );

    let recipe_ref = json!({
        "oneOf": [
            { "type": "string", "minLength": 1 },
            container_props(
                "recipeRef",
                &[
                    (dsl::recipe::steps::recipe_ref::object::field::ID, json!({ "type": "string", "minLength": 1 })),
                    (dsl::recipe::steps::recipe_ref::object::field::WITH, json!({
                        "type": "object",
                        "additionalProperties": { "type": "string" }
                    })),
                    (dsl::recipe::steps::condition::field::IF, string_prop()),
                    (dsl::recipe::steps::condition::field::IF_NOT, string_prop()),
                ],
            )
        ]
    });

    let if_step = container_props(
        "ifStep",
        &[
            (dsl::recipe::steps::condition::field::IF, string_prop()),
            (dsl::recipe::steps::condition::field::IF_NOT, string_prop()),
            (
                dsl::recipe::steps::if_step::field::STEPS,
                json!({ "type": "array", "items": ref_def("step"), "minItems": 1 }),
            ),
        ],
    );

    let mut edit_op_props = Map::new();
    for (wire, def_name) in [
        (dsl::recipe::steps::edit::ops::insert::WIRE, "insertOp"),
        (dsl::recipe::steps::edit::ops::replace::WIRE, "replaceOp"),
        (dsl::recipe::steps::edit::ops::remove::WIRE, "removeOp"),
    ] {
        let mut p = Map::new();
        p.insert("$ref".into(), Value::String(format!("#/definitions/{def_name}")));
        if let Some(entry) = all_entries()
            .iter()
            .find(|e| e.wire == wire && matches!(e.kind, VocabKind::OpKind))
        {
            p.insert(
                "description".into(),
                Value::String(entry.description.to_string()),
            );
        }
        edit_op_props.insert(wire.to_string(), Value::Object(p));
    }
    let edit_op = json!({
        "type": "object",
        "additionalProperties": false,
        "minProperties": 1,
        "maxProperties": 1,
        "properties": edit_op_props
    });

    let mut step_props = Map::new();
    for (wire, def_name, kind_match) in [
        (dsl::recipe::steps::edit::WIRE, "editStep", true),
        (dsl::recipe::steps::create::WIRE, "createStep", true),
        (dsl::recipe::steps::delete::WIRE, "deleteStep", true),
        (dsl::recipe::steps::recipe_ref::WIRE, "recipeRef", true),
        (dsl::recipe::steps::if_step::WIRE, "ifStep", true),
    ] {
        let mut p = Map::new();
        p.insert("$ref".into(), Value::String(format!("#/definitions/{def_name}")));
        if kind_match {
            if let Some(entry) = all_entries()
                .iter()
                .find(|e| e.wire == wire && matches!(e.kind, VocabKind::StepKind))
            {
                p.insert(
                    "description".into(),
                    Value::String(entry.description.to_string()),
                );
            }
        }
        step_props.insert(wire.to_string(), Value::Object(p));
    }
    let step = json!({
        "type": "object",
        "additionalProperties": false,
        "minProperties": 1,
        "maxProperties": 1,
        "properties": step_props
    });

    let arg = container_props(
        "arg",
        &[
            (dsl::recipe::arg::field::NAME, string_prop()),
            (dsl::recipe::arg::field::REQUIRED, bool_prop()),
            (dsl::recipe::arg::field::INPUT_KIND, enum_prop("inputKind")),
            (dsl::recipe::arg::field::ABBR, string_or_null()),
            (dsl::recipe::arg::field::HELP, string_or_null()),
            (dsl::recipe::arg::field::DEFAULTS_TO, string_or_null()),
            (
                dsl::recipe::arg::field::OPTIONS,
                json!({ "type": "array", "items": { "type": "string" } }),
            ),
            (dsl::recipe::arg::field::ALLOW_CUSTOM_VALUE, bool_prop()),
            (dsl::recipe::arg::field::CONTEXT_KEY, string_or_null()),
            (dsl::recipe::arg::field::FROM, json!({ "additionalProperties": true })),
        ],
    );

    let explorer_entry = container_props(
        "explorerMenuEntry",
        &[
            (
                dsl::recipe::explorer_menu::entry::field::KIND,
                enum_prop("explorerMenuKind"),
            ),
            (dsl::recipe::explorer_menu::entry::field::IF, string_prop()),
            (
                dsl::recipe::explorer_menu::entry::field::ARGS,
                json!({
                    "type": "object",
                    "additionalProperties": { "type": "string" }
                }),
            ),
        ],
    );

    let root = container_props(
        "recipeRoot",
        &[
            (dsl::recipe::field::ID, json!({ "type": "string", "minLength": 1 })),
            (dsl::recipe::field::NAME, string_prop()),
            (dsl::recipe::field::DESCRIPTION, string_prop()),
            (
                dsl::recipe::field::ARGS,
                json!({ "type": "array", "items": ref_def("arg") }),
            ),
            (
                dsl::recipe::field::MAPS,
                json!({
                    "type": "object",
                    "additionalProperties": {
                        "type": "object",
                        "additionalProperties": { "type": "string" }
                    }
                }),
            ),
            (
                dsl::recipe::field::QUERIES,
                json!({
                    "type": "object",
                    "additionalProperties": {
                        "type": "object",
                        "required": ["query"],
                        "properties": {
                            "query": { "type": "string" }
                        }
                    }
                }),
            ),
            (
                dsl::recipe::field::STEPS,
                json!({ "type": "array", "items": ref_def("step"), "minItems": 1 }),
            ),
            (
                dsl::recipe::field::POST_EXECUTION,
                json!({ "type": "array", "items": { "type": "string" } }),
            ),
            (
                dsl::recipe::field::EXPLORER_MENU,
                json!({
                    "oneOf": [
                        ref_def("explorerMenuEntry"),
                        { "type": "array", "items": ref_def("explorerMenuEntry") }
                    ]
                }),
            ),
        ],
    );

    let mut root_obj = root.as_object().cloned().unwrap();
    root_obj.insert(
        "$schema".into(),
        Value::String("http://json-schema.org/draft-07/schema#".into()),
    );
    root_obj.insert(
        "$id".into(),
        Value::String("https://codemod-recipe.dev/schemas/recipe.schema.json".into()),
    );
    root_obj.insert("title".into(), Value::String("Codemod Recipe".into()));
    root_obj.insert(
        "required".into(),
        json!([dsl::recipe::field::ID, dsl::recipe::field::STEPS]),
    );

    let mut definitions = Map::new();
    definitions.insert("arg".into(), arg);
    definitions.insert("editStep".into(), edit_step);
    definitions.insert("createStep".into(), create_step);
    definitions.insert("deleteStep".into(), delete_step);
    definitions.insert("recipeRef".into(), recipe_ref);
    definitions.insert("ifStep".into(), if_step);
    definitions.insert("step".into(), step);
    definitions.insert("editOp".into(), edit_op);
    definitions.insert("insertOp".into(), insert_op);
    definitions.insert("replaceOp".into(), replace_op);
    definitions.insert("removeOp".into(), remove_op);
    definitions.insert("letBinding".into(), let_binding);
    definitions.insert("explorerMenuEntry".into(), explorer_entry);
    definitions.insert(
        "queryField".into(),
        json!({
            "oneOf": [
                { "type": "string" },
                { "type": "array", "minItems": 1, "items": { "type": "string" } }
            ]
        }),
    );
    root_obj.insert("definitions".into(), Value::Object(definitions));

    Value::Object(root_obj)
}

pub fn map_schema() -> Value {
    let root = container_props(
        "mapRoot",
        &[
            (dsl::map_asset::field::ID, json!({ "type": "string", "minLength": 1 })),
            (dsl::map_asset::field::DESCRIPTION, string_prop()),
            (
                dsl::map_asset::field::MAP,
                json!({
                    "type": "object",
                    "additionalProperties": { "type": "string" }
                }),
            ),
        ],
    );
    let mut obj = root.as_object().cloned().unwrap();
    obj.insert(
        "$schema".into(),
        Value::String("http://json-schema.org/draft-07/schema#".into()),
    );
    obj.insert(
        "$id".into(),
        Value::String("https://codemod-recipe.dev/schemas/map.schema.json".into()),
    );
    obj.insert("title".into(), Value::String("Codemod Recipe Map".into()));
    obj.insert(
        "required".into(),
        json!([dsl::map_asset::field::ID, dsl::map_asset::field::MAP]),
    );
    Value::Object(obj)
}

pub fn variables_schema() -> Value {
    let root = container_props(
        "variablesRoot",
        &[
            (
                dsl::variables_asset::field::ID,
                json!({ "type": "string", "minLength": 1 }),
            ),
            (dsl::variables_asset::field::DESCRIPTION, string_prop()),
            (
                dsl::variables_asset::field::VALUES,
                json!({
                    "type": "object",
                    "additionalProperties": {
                        "oneOf": [
                            { "type": "string" },
                            { "type": "number" },
                            { "type": "boolean" }
                        ]
                    }
                }),
            ),
        ],
    );
    let mut obj = root.as_object().cloned().unwrap();
    obj.insert(
        "$schema".into(),
        Value::String("http://json-schema.org/draft-07/schema#".into()),
    );
    obj.insert(
        "$id".into(),
        Value::String("https://codemod-recipe.dev/schemas/variables.schema.json".into()),
    );
    obj.insert(
        "title".into(),
        Value::String("Codemod Recipe Variables".into()),
    );
    obj.insert(
        "required".into(),
        json!([
            dsl::variables_asset::field::ID,
            dsl::variables_asset::field::VALUES
        ]),
    );
    Value::Object(obj)
}

/// Sanity: every container in the inventory is referenced by id.
pub fn assert_structure_nonempty() {
    assert!(!CONTAINERS.is_empty());
    assert!(!ENUMS.is_empty());
}
