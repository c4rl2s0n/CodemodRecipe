use codemod_recipe_yaml::{
    container_by_id, container_for_parent_wire, dsl_surface_json, recipe_schema, CONTAINERS,
};
use codemod_recipe_yaml::dsl;

#[test]
fn edit_container_lists_model_fields() {
    let edit = container_by_id("edit").expect("edit");
    assert!(edit.children.contains(&dsl::recipe::steps::edit::field::PATH));
    assert!(edit.children.contains(&dsl::recipe::steps::edit::field::OPS));
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
fn surface_json_includes_all_containers() {
    let surface = dsl_surface_json();
    let containers = surface["containers"].as_object().unwrap();
    for c in CONTAINERS {
        assert!(
            containers.contains_key(c.id),
            "missing container {}",
            c.id
        );
    }
    assert!(surface["enums"]["anchor"].as_array().unwrap().len() >= 2);
}

#[test]
fn recipe_schema_has_required_id_and_steps() {
    let schema = recipe_schema();
    let required = schema["required"].as_array().unwrap();
    assert!(required.iter().any(|v| v == "id"));
    assert!(required.iter().any(|v| v == "steps"));
    assert!(schema["definitions"]["editStep"].is_object());
}
