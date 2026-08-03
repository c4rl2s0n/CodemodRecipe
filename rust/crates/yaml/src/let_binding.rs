use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Schema, SchemaObject, SubschemaValidation};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

use crate::query_spec::QuerySpec;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "letBinding", rename_all = "camelCase")]
pub struct LetBinding {
    pub name: String,
    #[serde(default)]
    pub query: Option<QuerySpec>,
    #[serde(default)]
    pub capture: Option<String>,
    #[serde(default)]
    pub extract: LetExtract,
    #[serde(default)]
    pub on_no_match: LetOnNoMatch,
    #[serde(default)]
    pub on_many_matches: LetOnManyMatches,
    #[serde(default)]
    pub join: Option<String>,
    /// Optional template to compute final value from prior locals (and recipe args).
    #[serde(default, rename = "as")]
    pub r#as: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[schemars(rename_all = "lowercase")]
pub enum LetExtract {
    #[default]
    Text,
    Kind,
    Exists,
    Count,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[schemars(rename_all = "lowercase")]
pub enum LetOnNoMatch {
    #[default]
    Error,
    #[serde(rename = "use")]
    UseEmpty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[schemars(rename_all = "lowercase")]
pub enum LetOnManyMatches {
    #[default]
    Error,
    First,
    Join,
}

/// Deserialize `let:` as a single binding or a list.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LetBindings(pub Vec<LetBinding>);

impl<'de> Deserialize<'de> for LetBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum OneOrMany {
            One(LetBinding),
            Many(Vec<LetBinding>),
        }
        match OneOrMany::deserialize(deserializer)? {
            OneOrMany::One(b) => Ok(LetBindings(vec![b])),
            OneOrMany::Many(v) => Ok(LetBindings(v)),
        }
    }
}

impl JsonSchema for LetBindings {
    fn schema_name() -> String {
        "letBindings".to_string()
    }

    fn json_schema(gen: &mut SchemaGenerator) -> Schema {
        let binding = gen.subschema_for::<LetBinding>();
        SchemaObject {
            subschemas: Some(Box::new(SubschemaValidation {
                one_of: Some(vec![
                    SchemaObject {
                        instance_type: Some(InstanceType::Array.into()),
                        array: Some(Box::new(schemars::schema::ArrayValidation {
                            items: Some(schemars::schema::SingleOrVec::Single(Box::new(
                                binding.clone(),
                            ))),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }
                    .into(),
                    SchemaObject {
                        instance_type: Some(InstanceType::Object.into()),
                        object: Some(Box::new(schemars::schema::ObjectValidation {
                            additional_properties: Some(Box::new(binding)),
                            ..Default::default()
                        })),
                        ..Default::default()
                    }
                    .into(),
                ]),
                ..Default::default()
            })),
            ..Default::default()
        }
        .into()
    }
}
