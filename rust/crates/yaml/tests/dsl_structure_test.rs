use codemod_recipe_yaml::dsl;
use codemod_recipe_yaml::schema_gen::{object_property_keys, resolve_definition};
use codemod_recipe_yaml::{
    container_children, container_for_parent_wire, dsl_surface_json, recipe_schema,
};

#[test]
fn edit_schema_lists_model_fields() {
    let schema = recipe_schema();
    let edit = resolve_definition(&schema, "editStep").expect("editStep");
    let keys = object_property_keys(edit);
    assert!(keys.iter().any(|k| k == dsl::recipe::steps::edit::field::PATH));
    assert!(keys.iter().any(|k| k == dsl::recipe::steps::edit::field::OPS));
    assert!(keys
        .iter()
        .any(|k| k == dsl::recipe::steps::edit::field::WHEN_NOT));
}

#[test]
fn parent_wire_maps_to_containers() {
    assert_eq!(
        container_for_parent_wire(dsl::recipe::steps::edit::WIRE),
        Some("edit")
    );
    assert_eq!(
        container_for_parent_wire(dsl::recipe::steps::recipe_ref::object::field::WITH),
        Some("with")
    );
}

#[test]
fn surface_json_includes_model_containers() {
    let surface = dsl_surface_json();
    let containers = surface["containers"].as_object().unwrap();
    for id in [
        "recipeRoot",
        "edit",
        "create",
        "delete",
        "recipeRef",
        "ifStep",
        "stepsItem",
        "opsItem",
        "insert",
        "replace",
        "remove",
        "arg",
        "letBinding",
        "with",
        "mapRoot",
        "variablesRoot",
    ] {
        assert!(containers.contains_key(id), "missing container {id}");
    }
    assert!(surface["enums"]["anchor"].as_array().unwrap().len() >= 2);
    let edit_children = container_children(&surface, "edit").expect("edit");
    assert!(edit_children.iter().any(|k| k == "path"));
    assert!(edit_children.iter().any(|k| k == "ops"));
}

#[test]
fn recipe_schema_has_required_id_and_steps() {
    let schema = recipe_schema();
    let required = schema["required"].as_array().unwrap();
    assert!(required.iter().any(|v| v == "id"));
    assert!(required.iter().any(|v| v == "steps"));
    assert!(schema["definitions"]["editStep"].is_object()
        || schema["$defs"]["editStep"].is_object());
}

#[test]
fn recipe_ref_surface_has_scalar_alt() {
    let surface = dsl_surface_json();
    assert_eq!(surface["containers"]["recipeRef"]["scalarAlt"], true);
    assert_eq!(surface["containers"]["with"]["mapValue"], true);
}
