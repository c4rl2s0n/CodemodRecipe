//! Emit `generated-dsl-surface.json` from schemars-backed JSON Schema documents.

use crate::dsl;
use crate::schema_gen::{
    collect_field_enums, map_schema, object_branch_of_one_of, object_property_keys, recipe_schema,
    resolve_definition, variables_schema,
};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

#[derive(Serialize)]
struct SurfaceContainer {
    children: Vec<String>,
    #[serde(rename = "mapValue", skip_serializing_if = "std::ops::Not::not")]
    map_value: bool,
    #[serde(rename = "scalarAlt", skip_serializing_if = "std::ops::Not::not")]
    scalar_alt: bool,
}

fn container(children: Vec<String>, map_value: bool, scalar_alt: bool) -> Value {
    serde_json::to_value(SurfaceContainer {
        children,
        map_value,
        scalar_alt,
    })
    .expect("surface container")
}

fn props_container(schema: &Value) -> Value {
    container(object_property_keys(schema), false, false)
}

/// Full editor surface artifact derived from model schemas.
pub fn dsl_surface_json() -> Value {
    let recipe = recipe_schema();
    let map = map_schema();
    let variables = variables_schema();

    let mut containers = Map::new();
    containers.insert("recipeRoot".into(), props_container(&recipe));
    containers.insert("mapRoot".into(), props_container(&map));
    containers.insert("variablesRoot".into(), props_container(&variables));

    for (def, id, scalar_alt) in [
        ("arg", "arg", false),
        ("editStep", "edit", false),
        ("createStep", "create", false),
        ("deleteStep", "delete", false),
        ("ifStep", "ifStep", false),
        ("insertOp", "insert", false),
        ("replaceOp", "replace", false),
        ("removeOp", "remove", false),
        ("letBinding", "letBinding", false),
        ("explorerMenuEntry", "explorerMenuEntry", false),
    ] {
        let def_schema = resolve_definition(&recipe, def).unwrap_or_else(|| {
            panic!("recipe schema missing definition {def}");
        });
        let mut c = props_container(def_schema);
        if scalar_alt {
            if let Some(obj) = c.as_object_mut() {
                obj.insert("scalarAlt".into(), Value::Bool(true));
            }
        }
        containers.insert(id.into(), c);
    }

    // recipeRef: object branch of oneOf, marked scalar_alt.
    let recipe_ref_def = resolve_definition(&recipe, "recipeRef").expect("recipeRef");
    let recipe_ref_obj =
        object_branch_of_one_of(&recipe, recipe_ref_def).unwrap_or(recipe_ref_def);
    // Prefer recipeRefObject properties when oneOf points at a $ref.
    let recipe_ref_props = if let Some(obj) = resolve_definition(&recipe, "recipeRefObject") {
        obj
    } else {
        recipe_ref_obj
    };
    containers.insert(
        "recipeRef".into(),
        container(object_property_keys(recipe_ref_props), false, true),
    );

    let step = resolve_definition(&recipe, "step").expect("step");
    containers.insert(
        "stepsItem".into(),
        container(object_property_keys(step), false, false),
    );
    let edit_op = resolve_definition(&recipe, "editOp").expect("editOp");
    containers.insert(
        "opsItem".into(),
        container(object_property_keys(edit_op), false, false),
    );
    containers.insert("with".into(), container(vec![], true, false));

    let enums = collect_field_enums(&recipe);

    let mut parent_to_container = BTreeMap::new();
    parent_to_container.insert("".to_string(), "recipeRoot".to_string());
    let parent_wires: &[(&str, &str)] = &[
        (dsl::recipe::steps::edit::WIRE, "edit"),
        (dsl::recipe::steps::create::WIRE, "create"),
        (dsl::recipe::steps::delete::WIRE, "delete"),
        (dsl::recipe::steps::recipe_ref::WIRE, "recipeRef"),
        (dsl::recipe::steps::if_step::WIRE, "ifStep"),
        (dsl::recipe::steps::edit::ops::insert::WIRE, "insert"),
        (dsl::recipe::steps::edit::ops::replace::WIRE, "replace"),
        (dsl::recipe::steps::edit::ops::remove::WIRE, "remove"),
        (dsl::recipe::steps::recipe_ref::object::field::WITH, "with"),
        (dsl::recipe::field::STEPS, "stepsItem"),
        (dsl::recipe::steps::edit::field::OPS, "opsItem"),
        (dsl::recipe::field::ARGS, "arg"),
        (dsl::recipe::steps::edit::field::LET, "letBinding"),
        (dsl::recipe::field::EXPLORER_MENU, "explorerMenuEntry"),
    ];
    for (wire, id) in parent_wires {
        parent_to_container.insert((*wire).to_string(), (*id).to_string());
    }

    // Sanity: every mapped parent container exists.
    for id in parent_to_container.values() {
        assert!(
            containers.contains_key(id),
            "parentToContainer references missing container {id}"
        );
    }

    json!({
        "version": 1,
        "containers": containers,
        "enums": enums,
        "parentToContainer": parent_to_container,
        "documentRoots": {
            "recipe": "recipeRoot",
            "map": "mapRoot",
            "variables": "variablesRoot"
        }
    })
}

/// Look up a surface container by id from the generated surface value.
pub fn container_children(surface: &Value, id: &str) -> Option<Vec<String>> {
    let children = surface
        .pointer(&format!("/containers/{id}/children"))?
        .as_array()?;
    Some(
        children
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
    )
}
