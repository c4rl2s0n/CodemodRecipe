//! Arg `from` derivation specs (editor builtins, templates, tree-sitter queries).

use schemars::gen::SchemaGenerator;
use schemars::schema::{Schema, SchemaObject};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::let_binding::{LetExtract, LetOnManyMatches};
use crate::query_spec::QuerySpec;

/// How a recipe arg is derived from editor / buffer context.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ArgFrom {
    /// Builtin context key (`file`, `selection`, `word`, …) or legacy `contextKey`.
    Builtin(String),
    Spec(ArgFromSpec),
}

impl JsonSchema for ArgFrom {
    fn schema_name() -> String {
        "argFrom".to_string()
    }

    fn json_schema(_gen: &mut SchemaGenerator) -> Schema {
        // Open shape: builtin string or free-form object (matches host validation leniency).
        SchemaObject {
            object: Some(Box::new(schemars::schema::ObjectValidation {
                additional_properties: Some(Box::new(Schema::Bool(true))),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "argFromSpec", rename_all = "camelCase")]
pub struct ArgFromSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub query: Option<QuerySpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capture: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extract: Option<LetExtract>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<ArgFromScope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default, rename = "as", skip_serializing_if = "Option::is_none")]
    pub r#as: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_no_match: Option<ArgFromOnNoMatch>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub on_many_matches: Option<LetOnManyMatches>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub join: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[schemars(rename_all = "lowercase")]
pub enum ArgFromScope {
    #[default]
    Enclosing,
    Selection,
    First,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Default, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[schemars(rename_all = "lowercase")]
pub enum ArgFromOnNoMatch {
    #[default]
    Omit,
    Empty,
}
