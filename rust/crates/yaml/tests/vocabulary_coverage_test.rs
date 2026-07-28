use codemod_recipe_yaml::all_entries;
use codemod_recipe_yaml::dsl;
use codemod_recipe_yaml::VocabKind;

/// Key and enum wires registered in ENTRIES (one representative per schema context).
#[test]
fn dsl_key_wires_have_registry_entries() {
    let wires = [
        dsl::recipe::field::ID,
        dsl::recipe::field::STEPS,
        dsl::map_asset::field::MAP,
        dsl::variables_asset::field::VALUES,
        dsl::recipe::steps::edit::WIRE,
        dsl::recipe::steps::edit::ops::insert::WIRE,
        dsl::recipe::steps::edit::ops::insert::field::QUERY,
        dsl::recipe::steps::create::field::IF_EXISTS,
        dsl::recipe::arg::field::INPUT_KIND,
        dsl::recipe::steps::edit::let_binding::field::EXTRACT,
    ];
    for wire in wires {
        assert!(
            all_entries()
                .iter()
                .any(|e| e.wire == wire && e.parent.is_none()),
            "missing vocabulary entry for dsl wire {wire}"
        );
    }
}

#[test]
fn dsl_enum_wires_have_registry_entries() {
    let cases = [
        (
            dsl::recipe::steps::create::field::IF_EXISTS,
            dsl::recipe::steps::create::field::if_exists::value::SKIP,
        ),
        (
            dsl::recipe::steps::edit::ops::insert::field::ANCHOR,
            dsl::recipe::steps::edit::ops::insert::field::anchor::value::END,
        ),
        (
            dsl::recipe::steps::edit::let_binding::field::EXTRACT,
            dsl::recipe::steps::edit::let_binding::field::extract::value::TEXT,
        ),
        (
            dsl::recipe::arg::field::INPUT_KIND,
            dsl::recipe::arg::field::input_kind::value::CHOICE,
        ),
    ];
    for (parent, value) in cases {
        assert!(
            all_entries()
                .iter()
                .any(|e| e.parent == Some(parent) && e.wire == value),
            "missing enum entry for {parent}.{value}"
        );
    }
}

#[test]
fn step_and_op_kinds_are_classified() {
    for (wire, kind) in [
        (dsl::recipe::steps::edit::WIRE, VocabKind::StepKind),
        (dsl::recipe::steps::create::WIRE, VocabKind::StepKind),
        (
            dsl::recipe::steps::edit::ops::insert::WIRE,
            VocabKind::OpKind,
        ),
        (
            dsl::recipe::steps::edit::ops::replace::WIRE,
            VocabKind::OpKind,
        ),
    ] {
        assert!(
            all_entries()
                .iter()
                .any(|e| e.wire == wire && e.kind == kind),
            "{wire}"
        );
    }
}

#[test]
fn entries_include_schema_path_for_recipe_root_id() {
    let id = all_entries()
        .iter()
        .find(|e| e.wire == dsl::recipe::field::ID && e.parent.is_none())
        .expect("recipe id entry");
    assert_eq!(id.schema_path, Some("#/properties/id"));
}
