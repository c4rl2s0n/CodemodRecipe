//! Parent-wire helpers for editor surface (derived at codegen time from model schemas).
//!
//! Structural vocabulary lives on [`crate::model`] via schemars. This module only
//! exposes the stable parent-wire → container-id map used by tests and tooling.

use crate::dsl;

/// Maps a YAML parent wire (indent parent key) to a container id for completions.
pub fn container_for_parent_wire(wire: &str) -> Option<&'static str> {
    match wire {
        w if w == dsl::recipe::steps::edit::WIRE => Some("edit"),
        w if w == dsl::recipe::steps::create::WIRE => Some("create"),
        w if w == dsl::recipe::steps::delete::WIRE => Some("delete"),
        w if w == dsl::recipe::steps::recipe_ref::WIRE => Some("recipeRef"),
        w if w == dsl::recipe::steps::if_step::WIRE => Some("ifStep"),
        w if w == dsl::recipe::steps::edit::ops::insert::WIRE => Some("insert"),
        w if w == dsl::recipe::steps::edit::ops::replace::WIRE => Some("replace"),
        w if w == dsl::recipe::steps::edit::ops::remove::WIRE => Some("remove"),
        w if w == dsl::recipe::steps::recipe_ref::object::field::WITH => Some("with"),
        w if w == dsl::recipe::field::STEPS => Some("stepsItem"),
        w if w == dsl::recipe::steps::edit::field::OPS => Some("opsItem"),
        w if w == dsl::recipe::field::ARGS => Some("arg"),
        w if w == dsl::recipe::steps::edit::field::LET => Some("letBinding"),
        w if w == dsl::recipe::field::EXPLORER_MENU => Some("explorerMenuEntry"),
        _ => None,
    }
}
