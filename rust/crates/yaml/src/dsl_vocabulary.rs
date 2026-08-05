//! Canonical codemod-recipe YAML DSL vocabulary (wire names, descriptions, enum values).
//!
//! ## Where things live
//!
//! - **Wire constants** (schema-shaped paths): [`crate::dsl`]
//! - **Author-facing prose** (VS Code hovers, JSON Schema `description`, TextMate): [`ENTRIES`] only — do not duplicate long docs on `dsl` consts
//! - **Optional `schema_path`** on each entry disambiguates duplicate wires (e.g. `query` on ops vs recipe `queries` map)
//!
//! After changing entries or `dsl` wires, run `scripts/generate-dsl-artifacts.sh`
//! (refreshes JSON Schema, keyword docs, TextMate, and `docs/generated/dsl-vocabulary.md`).

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum VocabKind {
    TopLevelField,
    AssetRootField,
    StepKind,
    OpKind,
    Field,
    EnumValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxGroup {
    StepKind,
    OpKind,
    FieldKey,
}

impl VocabKind {
    pub const fn syntax_group(self) -> Option<SyntaxGroup> {
        match self {
            Self::StepKind => Some(SyntaxGroup::StepKind),
            Self::OpKind => Some(SyntaxGroup::OpKind),
            Self::TopLevelField | Self::AssetRootField | Self::Field => Some(SyntaxGroup::FieldKey),
            Self::EnumValue => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VocabEntry {
    pub kind: VocabKind,
    pub wire: &'static str,
    pub parent: Option<&'static str>,
    pub description: &'static str,
    pub schema_path: Option<&'static str>,
}

macro_rules! vocab_entries {
    (
        $(
            $kind:ident, $wire:expr, $parent:expr, $desc:literal $(, $schema:literal)?
        );* $(;)?
    ) => {
        pub const ENTRIES: &[VocabEntry] = &[
            $(VocabEntry {
                kind: VocabKind::$kind,
                wire: $wire,
                parent: $parent,
                description: $desc,
                schema_path: vocab_entries!(@schema $($schema)?),
            }),*
        ];
    };
    (@schema) => { None };
    (@schema $s:literal) => { Some($s) };
}

vocab_entries! {
    TopLevelField, crate::dsl::recipe::field::ID, None, "Unique recipe identifier.", "#/properties/id";
    TopLevelField, crate::dsl::recipe::field::NAME, None, "Human-readable recipe name.", "#/properties/name";
    TopLevelField, crate::dsl::recipe::field::DESCRIPTION, None, "Short recipe description.", "#/properties/description";
    TopLevelField, crate::dsl::recipe::field::ARGS, None, "Recipe argument definitions shown in the runner UI.", "#/properties/args";
    TopLevelField, crate::dsl::recipe::field::MAPS, None, "Recipe-local map entries merged with workspace maps.", "#/properties/maps";
    TopLevelField, crate::dsl::recipe::field::QUERIES, None, "Recipe-local named query definitions.", "#/properties/queries";
    TopLevelField, crate::dsl::recipe::field::STEPS, None, "Ordered list of edit, create, delete, recipe, or if group steps.", "#/properties/steps";
    TopLevelField, crate::dsl::recipe::field::POST_EXECUTION, None, "Post-apply shell commands or script paths under the codemod root (Jinja-rendered).", "#/properties/postExecution";
    TopLevelField, crate::dsl::recipe::field::EXPLORER_MENU, None, "Opt-in for the VS Code Explorer Codemod Recipe submenu: list of { kind: file|folder, if?, args? } (single object is sugar). args maps recipe arg names to MiniJinja expressions over path; first matching entry wins.", "#/properties/explorerMenu";
    Field, crate::dsl::recipe::explorer_menu::entry::field::KIND, Some(crate::dsl::recipe::field::EXPLORER_MENU), "Explorer click kind this menu entry applies to.", "#/definitions/explorerMenuEntry/properties/kind";
    Field, crate::dsl::recipe::explorer_menu::entry::field::ARGS, Some(crate::dsl::recipe::field::EXPLORER_MENU), "Map of recipe arg name to MiniJinja expression over the Explorer click path (RHS sees only path + filters).", "#/definitions/explorerMenuEntry/properties/args";
    EnumValue, crate::dsl::recipe::explorer_menu::entry::field::kind::value::FILE, Some(crate::dsl::recipe::explorer_menu::entry::field::KIND), "Show when the Explorer selection is a file.";
    EnumValue, crate::dsl::recipe::explorer_menu::entry::field::kind::value::FOLDER, Some(crate::dsl::recipe::explorer_menu::entry::field::KIND), "Show when the Explorer selection is a folder.";

    AssetRootField, crate::dsl::map_asset::field::ID, None, "Unique map asset identifier.", "map#/properties/id";
    AssetRootField, crate::dsl::map_asset::field::DESCRIPTION, None, "Short map asset description.", "map#/properties/description";
    AssetRootField, crate::dsl::map_asset::field::MAP, None, "Map lookup table for a workspace map YAML asset (requires id).", "map#/properties/map";
    AssetRootField, crate::dsl::variables_asset::field::ID, None, "Unique variables asset identifier.", "variables#/properties/id";
    AssetRootField, crate::dsl::variables_asset::field::DESCRIPTION, None, "Short variables asset description.", "variables#/properties/description";
    AssetRootField, crate::dsl::variables_asset::field::VALUES, None, "Constant values for a workspace variables YAML asset (requires id).", "variables#/properties/values";

    StepKind, crate::dsl::recipe::steps::edit::WIRE, None, "Patch an existing file with tree-sitter insert, replace, or remove ops.";
    StepKind, crate::dsl::recipe::steps::create::WIRE, None, "Create a new file from inline template or templateFile.";
    StepKind, crate::dsl::recipe::steps::delete::WIRE, None, "Delete a file from the workspace.";
    StepKind, crate::dsl::recipe::steps::recipe_ref::WIRE, None, "Inline another recipe by id, optionally with call-site with bindings.";
    StepKind, crate::dsl::recipe::steps::if_step::WIRE, None, "Run nested steps when shared if / ifNot MiniJinja expressions pass.";

    OpKind, crate::dsl::recipe::steps::edit::ops::insert::WIRE, None, "Insert text at a capture anchor (start or end).";
    OpKind, crate::dsl::recipe::steps::edit::ops::replace::WIRE, None, "Replace the span of a query capture with new text.";
    OpKind, crate::dsl::recipe::steps::edit::ops::remove::WIRE, None, "Remove the span matched by a query capture.";

    Field, crate::dsl::recipe::steps::condition::field::IF, None, "MiniJinja expression over recipe args; skip the step when false.", "#/definitions/editStep/properties/if";
    Field, crate::dsl::recipe::steps::condition::field::IF_NOT, None, "MiniJinja expression over recipe args; skip the step when true.", "#/definitions/editStep/properties/ifNot";
    Field, crate::dsl::recipe::steps::if_step::field::STEPS, Some(crate::dsl::recipe::steps::if_step::WIRE), "Nested steps gated by the enclosing if step.", "#/definitions/ifStep/properties/steps";

    Field, crate::dsl::recipe::steps::edit::field::PATH, None, "Workspace-relative file path (often templated).", "#/definitions/editStep/properties/path";
    Field, crate::dsl::recipe::steps::edit::field::LANGUAGE, None, "Tree-sitter language id when extension inference is ambiguous.", "#/definitions/editStep/properties/language";
    Field, crate::dsl::recipe::steps::edit::field::WHEN, None, "Guard queries; all must match before the edit runs.", "#/definitions/editStep/properties/when";
    Field, crate::dsl::recipe::steps::edit::field::WHEN_NOT, None, "Forbidden guard queries; edit runs only if none match.", "#/definitions/editStep/properties/whenNot";
    Field, crate::dsl::recipe::steps::edit::field::LET, None, "Step-local bindings recomputed before each op.", "#/definitions/editStep/properties/let";
    Field, crate::dsl::recipe::steps::edit::field::OPS, None, "Ordered edit operations applied sequentially on the same file.", "#/definitions/editStep/properties/ops";

    Field, crate::dsl::recipe::steps::edit::ops::insert::field::QUERY, None, "Inline tree-sitter query, .scm path, query-library ref, or chain.", "#/definitions/insertOp/properties/query";
    Field, crate::dsl::recipe::steps::edit::ops::insert::field::CAPTURE, None, "Capture name from the query whose span is edited.", "#/definitions/insertOp/properties/capture";
    Field, crate::dsl::recipe::steps::edit::ops::insert::field::ANCHOR, None, "Insertion anchor relative to the capture (start or end).", "#/definitions/insertOp/properties/anchor";
    Field, crate::dsl::recipe::steps::edit::ops::insert::field::TEXT, None, "Replacement or inserted source text (Jinja-rendered).", "#/definitions/insertOp/properties/text";

    Field, crate::dsl::recipe::steps::edit::ops::replace::field::QUERY, None, "Inline tree-sitter query, .scm path, query-library ref, or chain.", "#/definitions/replaceOp/properties/query";
    Field, crate::dsl::recipe::steps::edit::ops::replace::field::CAPTURE, None, "Capture name from the query whose span is edited.", "#/definitions/replaceOp/properties/capture";
    Field, crate::dsl::recipe::steps::edit::ops::replace::field::TEXT, None, "Replacement or inserted source text (Jinja-rendered).", "#/definitions/replaceOp/properties/text";
    Field, crate::dsl::recipe::steps::edit::ops::replace::field::INCLUDE_LEADING_TRIVIA, None, "Include leading trivia when replacing or removing a capture.", "#/definitions/replaceOp/properties/includeLeadingTrivia";

    Field, crate::dsl::recipe::steps::edit::ops::remove::field::QUERY, None, "Inline tree-sitter query, .scm path, query-library ref, or chain.", "#/definitions/removeOp/properties/query";
    Field, crate::dsl::recipe::steps::edit::ops::remove::field::CAPTURE, None, "Capture name from the query whose span is edited.", "#/definitions/removeOp/properties/capture";
    Field, crate::dsl::recipe::steps::edit::ops::remove::field::INCLUDE_LEADING_TRIVIA, None, "Include leading trivia when replacing or removing a capture.", "#/definitions/removeOp/properties/includeLeadingTrivia";

    Field, crate::dsl::recipe::steps::create::field::PATH, None, "Workspace-relative file path (often templated).", "#/definitions/createStep/properties/path";
    Field, crate::dsl::recipe::steps::create::field::TEMPLATE, None, "Inline file content for a create step.", "#/definitions/createStep/properties/template";
    Field, crate::dsl::recipe::steps::create::field::TEMPLATE_FILE, None, "Template path resolved recipe-local first, then under .codemod.", "#/definitions/createStep/properties/templateFile";
    Field, crate::dsl::recipe::steps::create::field::IF_EXISTS, None, "Behavior when create target path already exists.", "#/definitions/createStep/properties/ifExists";

    Field, crate::dsl::recipe::steps::delete::field::PATH, None, "Workspace-relative file path (often templated).", "#/definitions/deleteStep/properties/path";
    Field, crate::dsl::recipe::steps::delete::field::IF_MISSING, None, "Behavior when delete target path is missing.", "#/definitions/deleteStep/properties/ifMissing";

    Field, crate::dsl::recipe::steps::recipe_ref::object::field::WITH, None, "Call-site arg bindings when referencing a child recipe.", "#/definitions/recipeRef/properties/with";

    Field, crate::dsl::recipe::arg::field::REQUIRED, None, "Whether the recipe arg must be provided.", "#/definitions/arg/properties/required";
    Field, crate::dsl::recipe::arg::field::INPUT_KIND, None, "Runner input widget kind for a recipe arg.", "#/definitions/arg/properties/inputKind";
    Field, crate::dsl::recipe::arg::field::ABBR, None, "Short CLI abbreviation for a recipe arg.", "#/definitions/arg/properties/abbr";
    Field, crate::dsl::recipe::arg::field::HELP, None, "Help text shown in the recipe runner.", "#/definitions/arg/properties/help";
    Field, crate::dsl::recipe::arg::field::DEFAULTS_TO, None, "Default value when the arg is omitted.", "#/definitions/arg/properties/defaultsTo";
    Field, crate::dsl::recipe::arg::field::OPTIONS, None, "Allowed values for choice-style args.", "#/definitions/arg/properties/options";
    Field, crate::dsl::recipe::arg::field::ALLOW_CUSTOM_VALUE, None, "Allow values outside options for choice args.", "#/definitions/arg/properties/allowCustomValue";
    Field, crate::dsl::recipe::arg::field::CONTEXT_KEY, None, "Deprecated alias of string `from` — editor context key used to prefill this arg.", "#/definitions/arg/properties/contextKey";
    Field, crate::dsl::recipe::arg::field::FROM, None, "How to derive this arg from editor context: builtin key, template, or tree-sitter query (like let).", "#/definitions/arg/properties/from";
    Field, crate::dsl::recipe::steps::recipe_ref::object::field::ID, None, "Unique recipe identifier for an inlined recipe reference.", "#/definitions/recipeRef/properties/id";

    Field, crate::dsl::recipe::arg::field::NAME, None, "Recipe argument name.", "#/definitions/arg/properties/name";

    Field, crate::dsl::recipe::steps::edit::let_binding::field::EXTRACT, None, "How to derive a let binding value from query captures.", "#/definitions/letBinding/properties/extract";
    Field, crate::dsl::recipe::steps::edit::let_binding::field::ON_NO_MATCH, None, "Behavior when a let binding query matches nothing.", "#/definitions/letBinding/properties/onNoMatch";
    Field, crate::dsl::recipe::steps::edit::let_binding::field::ON_MANY_MATCHES, None, "Behavior when a let binding query matches multiple nodes.", "#/definitions/letBinding/properties/onManyMatches";
    Field, crate::dsl::recipe::steps::edit::let_binding::field::JOIN, None, "Separator when onManyMatches is join.", "#/definitions/letBinding/properties/join";
    Field, crate::dsl::recipe::steps::edit::let_binding::field::AS, None, "Jinja template to compute a let value from prior locals.", "#/definitions/letBinding/properties/as";
    Field, crate::dsl::recipe::steps::edit::let_binding::field::NAME, None, "Let binding local name.", "#/definitions/letBinding/properties/name";
    Field, crate::dsl::recipe::steps::edit::let_binding::field::QUERY, None, "Inline tree-sitter query, .scm path, query-library ref, or chain.", "#/definitions/letBinding/properties/query";
    Field, crate::dsl::recipe::steps::edit::let_binding::field::CAPTURE, None, "Capture name from the query whose span is edited.", "#/definitions/letBinding/properties/capture";

    Field, crate::dsl::recipe::queries::entry::field::QUERY, None, "Inline tree-sitter query, .scm path, query-library ref, or chain.", "#/properties/queries/additionalProperties/query";

    EnumValue, crate::dsl::recipe::steps::create::field::if_exists::value::FAIL, Some(crate::dsl::recipe::steps::create::field::IF_EXISTS), "Abort the recipe when the create path already exists.";
    EnumValue, crate::dsl::recipe::steps::create::field::if_exists::value::SKIP, Some(crate::dsl::recipe::steps::create::field::IF_EXISTS), "Skip the create step when the path already exists on disk or was staged earlier in this recipe run.";
    EnumValue, crate::dsl::recipe::steps::delete::field::if_missing::value::FAIL, Some(crate::dsl::recipe::steps::delete::field::IF_MISSING), "Abort when deleting a missing file.";
    EnumValue, crate::dsl::recipe::steps::delete::field::if_missing::value::SKIP, Some(crate::dsl::recipe::steps::delete::field::IF_MISSING), "Skip delete when the file is already absent.";
    EnumValue, crate::dsl::recipe::steps::edit::ops::insert::field::anchor::value::START, Some(crate::dsl::recipe::steps::edit::ops::insert::field::ANCHOR), "Insert before the capture span.";
    EnumValue, crate::dsl::recipe::steps::edit::ops::insert::field::anchor::value::END, Some(crate::dsl::recipe::steps::edit::ops::insert::field::ANCHOR), "Insert after the capture span.";
    EnumValue, crate::dsl::recipe::steps::edit::let_binding::field::extract::value::TEXT, Some(crate::dsl::recipe::steps::edit::let_binding::field::EXTRACT), "Capture source text.";
    EnumValue, crate::dsl::recipe::steps::edit::let_binding::field::extract::value::KIND, Some(crate::dsl::recipe::steps::edit::let_binding::field::EXTRACT), "Tree-sitter node kind name.";
    EnumValue, crate::dsl::recipe::steps::edit::let_binding::field::extract::value::EXISTS, Some(crate::dsl::recipe::steps::edit::let_binding::field::EXTRACT), "True when the capture matched.";
    EnumValue, crate::dsl::recipe::steps::edit::let_binding::field::extract::value::COUNT, Some(crate::dsl::recipe::steps::edit::let_binding::field::EXTRACT), "Number of capture matches.";
    EnumValue, crate::dsl::recipe::steps::edit::let_binding::field::on_no_match::value::ERROR, Some(crate::dsl::recipe::steps::edit::let_binding::field::ON_NO_MATCH), "Fail the edit when no match.";
    EnumValue, crate::dsl::recipe::steps::edit::let_binding::field::on_no_match::value::USE, Some(crate::dsl::recipe::steps::edit::let_binding::field::ON_NO_MATCH), "Use an empty value when no match.";
    EnumValue, crate::dsl::recipe::steps::edit::let_binding::field::on_many_matches::value::ERROR, Some(crate::dsl::recipe::steps::edit::let_binding::field::ON_MANY_MATCHES), "Fail when multiple matches.";
    EnumValue, crate::dsl::recipe::steps::edit::let_binding::field::on_many_matches::value::FIRST, Some(crate::dsl::recipe::steps::edit::let_binding::field::ON_MANY_MATCHES), "Use the first matching capture.";
    EnumValue, crate::dsl::recipe::steps::edit::let_binding::field::on_many_matches::value::JOIN, Some(crate::dsl::recipe::steps::edit::let_binding::field::ON_MANY_MATCHES), "Join all matches with join separator.";
    EnumValue, crate::dsl::recipe::arg::field::input_kind::value::TEXT, Some(crate::dsl::recipe::arg::field::INPUT_KIND), "Free-text arg input.";
    EnumValue, crate::dsl::recipe::arg::field::input_kind::value::FILE, Some(crate::dsl::recipe::arg::field::INPUT_KIND), "File path arg input.";
    EnumValue, crate::dsl::recipe::arg::field::input_kind::value::DIRECTORY, Some(crate::dsl::recipe::arg::field::INPUT_KIND), "Directory path arg input.";
    EnumValue, crate::dsl::recipe::arg::field::input_kind::value::CHOICE, Some(crate::dsl::recipe::arg::field::INPUT_KIND), "Pick from options list.";
}

pub fn all_entries() -> &'static [VocabEntry] {
    ENTRIES
}

pub fn description_for_key(wire: &str) -> Option<&'static str> {
    // Prefer field-like entries so step-kind wires that share a name (e.g. `if`)
    // do not override property descriptions on edit/create/ifStep bodies.
    ENTRIES
        .iter()
        .find(|e| {
            e.parent.is_none()
                && e.wire == wire
                && !matches!(e.kind, VocabKind::StepKind | VocabKind::OpKind)
        })
        .or_else(|| {
            ENTRIES
                .iter()
                .find(|e| e.parent.is_none() && e.wire == wire)
        })
        .map(|e| e.description)
}

pub fn description_for_enum(parent: &str, value: &str) -> Option<&'static str> {
    ENTRIES
        .iter()
        .find(|e| e.parent == Some(parent) && e.wire == value)
        .map(|e| e.description)
}

pub fn syntax_alternation(group: SyntaxGroup) -> String {
    let mut wires: Vec<&str> = ENTRIES
        .iter()
        .filter(|e| e.kind.syntax_group() == Some(group))
        .map(|e| e.wire)
        .collect();
    wires.sort_unstable();
    wires.dedup();
    wires.join("|")
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeywordDocJson {
    pub kind: String,
    pub wire: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_path: Option<String>,
    pub description: String,
}

pub fn keyword_docs_json() -> Vec<KeywordDocJson> {
    ENTRIES
        .iter()
        .map(|e| KeywordDocJson {
            kind: format!("{:?}", e.kind),
            wire: e.wire.to_string(),
            parent: e.parent.map(str::to_string),
            schema_path: e.schema_path.map(str::to_string),
            description: e.description.to_string(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dsl_wires_match_registry() {
        let pairs = [
            (crate::dsl::recipe::steps::edit::WIRE, "edit"),
            (crate::dsl::recipe::steps::create::WIRE, "create"),
            (
                crate::dsl::recipe::steps::edit::ops::insert::field::QUERY,
                "query",
            ),
            (crate::dsl::recipe::field::POST_EXECUTION, "postExecution"),
        ];
        for (constant, wire) in pairs {
            assert_eq!(constant, wire);
            assert!(description_for_key(wire).is_some());
        }
    }

    #[test]
    fn enum_values_have_descriptions() {
        assert!(description_for_enum(
            crate::dsl::recipe::steps::create::field::IF_EXISTS,
            crate::dsl::recipe::steps::create::field::if_exists::value::SKIP
        )
        .is_some());
        assert!(description_for_enum(
            crate::dsl::recipe::steps::edit::ops::insert::field::ANCHOR,
            crate::dsl::recipe::steps::edit::ops::insert::field::anchor::value::END
        )
        .is_some());
    }

    #[test]
    fn step_kinds_map_to_step_syntax_group() {
        for wire in [
            crate::dsl::recipe::steps::edit::WIRE,
            crate::dsl::recipe::steps::create::WIRE,
            crate::dsl::recipe::steps::delete::WIRE,
            crate::dsl::recipe::steps::recipe_ref::WIRE,
            crate::dsl::recipe::steps::if_step::WIRE,
        ] {
            let entry = ENTRIES
                .iter()
                .find(|e| e.wire == wire && e.kind == VocabKind::StepKind)
                .expect(wire);
            assert_eq!(entry.kind.syntax_group(), Some(SyntaxGroup::StepKind));
        }
    }
}
