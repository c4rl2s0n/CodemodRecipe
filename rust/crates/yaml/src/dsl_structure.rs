//! Structural inventory of the YAML DSL, aligned with [`crate::model`].
//!
//! Codegen emits JSON Schema and `generated-dsl-surface.json` from this module
//! so editor tooling does not hand-maintain a second container graph.
//! When adding fields to model structs, update the matching container here
//! (and `ENTRIES` prose / `dsl::` wires as today).

use crate::dsl;

/// A named object shape (recipe root, edit step, insert op, …).
#[derive(Debug, Clone, Copy)]
pub struct ContainerDef {
    pub id: &'static str,
    /// Child property wires (YAML keys).
    pub children: &'static [&'static str],
    /// When true, values are an open string→string map (e.g. `with:`).
    pub map_value: bool,
    /// When true, the value may also be a scalar string (e.g. recipe ref id).
    pub scalar_alt: bool,
}

/// Static string enums keyed by field wire (or `parent.field` when needed).
#[derive(Debug, Clone, Copy)]
pub struct EnumDef {
    pub id: &'static str,
    pub values: &'static [&'static str],
}

pub const CONTAINERS: &[ContainerDef] = &[
    ContainerDef {
        id: "recipeRoot",
        children: &[
            dsl::recipe::field::ID,
            dsl::recipe::field::NAME,
            dsl::recipe::field::DESCRIPTION,
            dsl::recipe::field::ARGS,
            dsl::recipe::field::MAPS,
            dsl::recipe::field::QUERIES,
            dsl::recipe::field::STEPS,
            dsl::recipe::field::POST_EXECUTION,
            dsl::recipe::field::EXPLORER_MENU,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "mapRoot",
        children: &[
            dsl::map_asset::field::ID,
            dsl::map_asset::field::DESCRIPTION,
            dsl::map_asset::field::MAP,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "variablesRoot",
        children: &[
            dsl::variables_asset::field::ID,
            dsl::variables_asset::field::DESCRIPTION,
            dsl::variables_asset::field::VALUES,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "arg",
        children: &[
            dsl::recipe::arg::field::NAME,
            dsl::recipe::arg::field::REQUIRED,
            dsl::recipe::arg::field::INPUT_KIND,
            dsl::recipe::arg::field::ABBR,
            dsl::recipe::arg::field::HELP,
            dsl::recipe::arg::field::DEFAULTS_TO,
            dsl::recipe::arg::field::OPTIONS,
            dsl::recipe::arg::field::ALLOW_CUSTOM_VALUE,
            dsl::recipe::arg::field::CONTEXT_KEY,
            dsl::recipe::arg::field::FROM,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "stepsItem",
        children: &[
            dsl::recipe::steps::edit::WIRE,
            dsl::recipe::steps::create::WIRE,
            dsl::recipe::steps::delete::WIRE,
            dsl::recipe::steps::recipe_ref::WIRE,
            dsl::recipe::steps::if_step::WIRE,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "edit",
        children: &[
            dsl::recipe::steps::edit::field::PATH,
            dsl::recipe::steps::edit::field::LANGUAGE,
            dsl::recipe::steps::edit::field::WHEN,
            dsl::recipe::steps::edit::field::WHEN_NOT,
            dsl::recipe::steps::edit::field::LET,
            dsl::recipe::steps::edit::field::OPS,
            dsl::recipe::steps::condition::field::IF,
            dsl::recipe::steps::condition::field::IF_NOT,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "create",
        children: &[
            dsl::recipe::steps::create::field::PATH,
            dsl::recipe::steps::create::field::TEMPLATE,
            dsl::recipe::steps::create::field::TEMPLATE_FILE,
            dsl::recipe::steps::create::field::IF_EXISTS,
            dsl::recipe::steps::condition::field::IF,
            dsl::recipe::steps::condition::field::IF_NOT,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "delete",
        children: &[
            dsl::recipe::steps::delete::field::PATH,
            dsl::recipe::steps::delete::field::IF_MISSING,
            dsl::recipe::steps::condition::field::IF,
            dsl::recipe::steps::condition::field::IF_NOT,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "recipeRef",
        children: &[
            dsl::recipe::steps::recipe_ref::object::field::ID,
            dsl::recipe::steps::recipe_ref::object::field::WITH,
            dsl::recipe::steps::condition::field::IF,
            dsl::recipe::steps::condition::field::IF_NOT,
        ],
        map_value: false,
        scalar_alt: true,
    },
    ContainerDef {
        id: "ifStep",
        children: &[
            dsl::recipe::steps::condition::field::IF,
            dsl::recipe::steps::condition::field::IF_NOT,
            dsl::recipe::steps::if_step::field::STEPS,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "opsItem",
        children: &[
            dsl::recipe::steps::edit::ops::insert::WIRE,
            dsl::recipe::steps::edit::ops::replace::WIRE,
            dsl::recipe::steps::edit::ops::remove::WIRE,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "insert",
        children: &[
            dsl::recipe::steps::edit::ops::insert::field::QUERY,
            dsl::recipe::steps::edit::ops::insert::field::CAPTURE,
            dsl::recipe::steps::edit::ops::insert::field::ANCHOR,
            dsl::recipe::steps::edit::ops::insert::field::TEXT,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "replace",
        children: &[
            dsl::recipe::steps::edit::ops::replace::field::QUERY,
            dsl::recipe::steps::edit::ops::replace::field::CAPTURE,
            dsl::recipe::steps::edit::ops::replace::field::TEXT,
            dsl::recipe::steps::edit::ops::replace::field::INCLUDE_LEADING_TRIVIA,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "remove",
        children: &[
            dsl::recipe::steps::edit::ops::remove::field::QUERY,
            dsl::recipe::steps::edit::ops::remove::field::CAPTURE,
            dsl::recipe::steps::edit::ops::remove::field::INCLUDE_LEADING_TRIVIA,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "letBinding",
        children: &[
            dsl::recipe::steps::edit::let_binding::field::NAME,
            dsl::recipe::steps::edit::let_binding::field::QUERY,
            dsl::recipe::steps::edit::let_binding::field::CAPTURE,
            dsl::recipe::steps::edit::let_binding::field::EXTRACT,
            dsl::recipe::steps::edit::let_binding::field::ON_NO_MATCH,
            dsl::recipe::steps::edit::let_binding::field::ON_MANY_MATCHES,
            dsl::recipe::steps::edit::let_binding::field::JOIN,
            dsl::recipe::steps::edit::let_binding::field::AS,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "explorerMenuEntry",
        children: &[
            dsl::recipe::explorer_menu::entry::field::KIND,
            dsl::recipe::explorer_menu::entry::field::IF,
            dsl::recipe::explorer_menu::entry::field::ARGS,
        ],
        map_value: false,
        scalar_alt: false,
    },
    ContainerDef {
        id: "with",
        children: &[],
        map_value: true,
        scalar_alt: false,
    },
];

pub const ENUMS: &[EnumDef] = &[
    EnumDef {
        id: "anchor",
        values: &[
            dsl::recipe::steps::edit::ops::insert::field::anchor::value::START,
            dsl::recipe::steps::edit::ops::insert::field::anchor::value::END,
        ],
    },
    EnumDef {
        id: "ifExists",
        values: &[
            dsl::recipe::steps::create::field::if_exists::value::FAIL,
            dsl::recipe::steps::create::field::if_exists::value::SKIP,
        ],
    },
    EnumDef {
        id: "ifMissing",
        values: &[
            dsl::recipe::steps::delete::field::if_missing::value::FAIL,
            dsl::recipe::steps::delete::field::if_missing::value::SKIP,
        ],
    },
    EnumDef {
        id: "inputKind",
        values: &[
            dsl::recipe::arg::field::input_kind::value::TEXT,
            dsl::recipe::arg::field::input_kind::value::FILE,
            dsl::recipe::arg::field::input_kind::value::DIRECTORY,
            dsl::recipe::arg::field::input_kind::value::CHOICE,
        ],
    },
    EnumDef {
        id: "extract",
        values: &[
            dsl::recipe::steps::edit::let_binding::field::extract::value::TEXT,
            dsl::recipe::steps::edit::let_binding::field::extract::value::KIND,
            dsl::recipe::steps::edit::let_binding::field::extract::value::EXISTS,
            dsl::recipe::steps::edit::let_binding::field::extract::value::COUNT,
        ],
    },
    EnumDef {
        id: "onNoMatch",
        values: &[
            dsl::recipe::steps::edit::let_binding::field::on_no_match::value::ERROR,
            dsl::recipe::steps::edit::let_binding::field::on_no_match::value::USE,
        ],
    },
    EnumDef {
        id: "onManyMatches",
        values: &[
            dsl::recipe::steps::edit::let_binding::field::on_many_matches::value::ERROR,
            dsl::recipe::steps::edit::let_binding::field::on_many_matches::value::FIRST,
            dsl::recipe::steps::edit::let_binding::field::on_many_matches::value::JOIN,
        ],
    },
    EnumDef {
        id: "explorerMenuKind",
        values: &[
            dsl::recipe::explorer_menu::entry::field::kind::value::FILE,
            dsl::recipe::explorer_menu::entry::field::kind::value::FOLDER,
        ],
    },
];

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

pub fn container_by_id(id: &str) -> Option<&'static ContainerDef> {
    CONTAINERS.iter().find(|c| c.id == id)
}
