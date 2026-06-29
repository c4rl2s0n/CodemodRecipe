use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Recipe {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub args: Vec<Arg>,
    #[serde(default)]
    pub maps: BTreeMap<String, BTreeMap<String, String>>,
    pub steps: Vec<Step>,
    #[serde(default, rename = "postExecution")]
    pub post_execution: Vec<PostExecution>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Arg {
    pub name: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default, rename = "inputKind")]
    pub input_kind: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum PostExecution {
    String(String),
    Map(serde_yaml::Value),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Edit(EditStep),
    Create(serde_yaml::Value),
    RecipeRef(serde_yaml::Value),
    Unknown(String, serde_yaml::Value),
}

impl<'de> Deserialize<'de> for Step {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct StepVisitor;

        impl<'de> Visitor<'de> for StepVisitor {
            type Value = Step;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with a single step key (edit/create/recipe/...)")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let (k, v): (String, serde_yaml::Value) = map
                    .next_entry()?
                    .ok_or_else(|| de::Error::custom("empty step map"))?;

                // Ensure single-key map.
                if map.next_entry::<String, serde_yaml::Value>()?.is_some() {
                    return Err(de::Error::custom("step map must have exactly one key"));
                }

                match k.as_str() {
                    "edit" => {
                        let edit: EditStep = serde_yaml::from_value(v)
                            .map_err(|e| de::Error::custom(format!("invalid edit step: {e}")))?;
                        Ok(Step::Edit(edit))
                    }
                    "create" => Ok(Step::Create(v)),
                    "recipe" => Ok(Step::RecipeRef(v)),
                    other => Ok(Step::Unknown(other.to_string(), v)),
                }
            }
        }

        deserializer.deserialize_map(StepVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct EditStep {
    pub path: String,
    #[serde(default)]
    pub language: Option<String>,
    pub ops: Vec<EditOp>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditOp {
    Insert(InsertOp),
    Replace(ReplaceOp),
    Remove(RemoveOp),
    Unknown(String, serde_yaml::Value),
}

impl<'de> Deserialize<'de> for EditOp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct OpVisitor;

        impl<'de> Visitor<'de> for OpVisitor {
            type Value = EditOp;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a map with a single op key (insert/replace/remove)")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let (k, v): (String, serde_yaml::Value) = map
                    .next_entry()?
                    .ok_or_else(|| de::Error::custom("empty op map"))?;

                if map.next_entry::<String, serde_yaml::Value>()?.is_some() {
                    return Err(de::Error::custom("op map must have exactly one key"));
                }

                match k.as_str() {
                    "insert" => {
                        let op: InsertOp = serde_yaml::from_value(v)
                            .map_err(|e| de::Error::custom(format!("invalid insert op: {e}")))?;
                        Ok(EditOp::Insert(op))
                    }
                    "replace" => {
                        let op: ReplaceOp = serde_yaml::from_value(v)
                            .map_err(|e| de::Error::custom(format!("invalid replace op: {e}")))?;
                        Ok(EditOp::Replace(op))
                    }
                    "remove" => {
                        let op: RemoveOp = serde_yaml::from_value(v)
                            .map_err(|e| de::Error::custom(format!("invalid remove op: {e}")))?;
                        Ok(EditOp::Remove(op))
                    }
                    other => Ok(EditOp::Unknown(other.to_string(), v)),
                }
            }
        }

        deserializer.deserialize_map(OpVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct InsertOp {
    pub query: String,
    pub capture: String,
    pub anchor: InsertAnchor,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InsertAnchor {
    Start,
    End,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ReplaceOp {
    pub query: String,
    pub capture: String,
    pub text: String,
    #[serde(default, rename = "includeLeadingTrivia")]
    pub include_leading_trivia: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RemoveOp {
    pub query: String,
    pub capture: String,
    #[serde(default, rename = "includeLeadingTrivia")]
    pub include_leading_trivia: bool,
}
