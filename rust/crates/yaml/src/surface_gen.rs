//! Emit `generated-dsl-surface.json` from [`crate::dsl_structure`].

use crate::dsl_structure::{container_for_parent_wire, CONTAINERS, ENUMS};
use crate::dsl;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

#[derive(Serialize)]
struct SurfaceContainer {
    children: Vec<&'static str>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    map_value: bool,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    scalar_alt: bool,
}

/// Full editor surface artifact.
pub fn dsl_surface_json() -> Value {
    let mut containers = Map::new();
    for c in CONTAINERS {
        let entry = SurfaceContainer {
            children: c.children.to_vec(),
            map_value: c.map_value,
            scalar_alt: c.scalar_alt,
        };
        containers.insert(
            c.id.to_string(),
            serde_json::to_value(entry).expect("surface container"),
        );
    }

    let mut enums = Map::new();
    for e in ENUMS {
        enums.insert(
            e.id.to_string(),
            Value::Array(
                e.values
                    .iter()
                    .map(|v| Value::String((*v).to_string()))
                    .collect(),
            ),
        );
    }

    // Parent wire → container id (for yamlContext resolution).
    let parent_wires = [
        dsl::recipe::steps::edit::WIRE,
        dsl::recipe::steps::create::WIRE,
        dsl::recipe::steps::delete::WIRE,
        dsl::recipe::steps::recipe_ref::WIRE,
        dsl::recipe::steps::if_step::WIRE,
        dsl::recipe::steps::edit::ops::insert::WIRE,
        dsl::recipe::steps::edit::ops::replace::WIRE,
        dsl::recipe::steps::edit::ops::remove::WIRE,
        dsl::recipe::steps::recipe_ref::object::field::WITH,
        dsl::recipe::field::STEPS,
        dsl::recipe::steps::edit::field::OPS,
        dsl::recipe::field::ARGS,
        dsl::recipe::steps::edit::field::LET,
        dsl::recipe::field::EXPLORER_MENU,
    ];
    let mut parent_to_container = BTreeMap::new();
    for wire in parent_wires {
        if let Some(id) = container_for_parent_wire(wire) {
            parent_to_container.insert(wire.to_string(), id.to_string());
        }
    }
    // Document root when no parent (top-level recipe keys).
    parent_to_container.insert("".to_string(), "recipeRoot".to_string());

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
