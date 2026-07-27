use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;

use std::collections::BTreeMap;

use crate::guard_list::GuardList;
use crate::let_binding::LetBindings;
pub use crate::query_spec::{QueryDefinition, QuerySpec};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Recipe {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    /// Optional dotted catalog path (e.g. `rust.data`, `dart.feature.states`).
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub args: Vec<Arg>,
    #[serde(default)]
    pub maps: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(default)]
    pub queries: BTreeMap<String, QueryDefinition>,
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
    #[serde(default)]
    pub abbr: Option<String>,
    #[serde(default)]
    pub help: Option<String>,
    #[serde(default, rename = "defaultsTo")]
    pub defaults_to: Option<String>,
    #[serde(default)]
    pub options: Vec<String>,
    #[serde(default, rename = "allowCustomValue")]
    pub allow_custom_value: Option<bool>,
    #[serde(default, rename = "contextKey")]
    pub context_key: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum PostExecution {
    String(String),
    Map(serde_yaml::Value),
}

/// Reference to another recipe, optionally with call-site arg bindings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeRef {
    pub id: String,
    /// Child arg name → template string rendered in the parent context.
    pub with: BTreeMap<String, String>,
}

/// Inlined child steps with call-site `with` overlays (produced by compose expand).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopedStep {
    pub with: BTreeMap<String, String>,
    pub steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step {
    Edit(EditStep),
    Create(CreateStep),
    Delete(DeleteStep),
    RecipeRef(RecipeRef),
    Scoped(ScopedStep),
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
                    "create" => {
                        let create: CreateStep = serde_yaml::from_value(v)
                            .map_err(|e| de::Error::custom(format!("invalid create step: {e}")))?;
                        Ok(Step::Create(create))
                    }
                    "delete" => {
                        let delete: DeleteStep = serde_yaml::from_value(v)
                            .map_err(|e| de::Error::custom(format!("invalid delete step: {e}")))?;
                        Ok(Step::Delete(delete))
                    }
                    "recipe" => {
                        let recipe_ref = parse_recipe_ref(v).map_err(de::Error::custom)?;
                        Ok(Step::RecipeRef(recipe_ref))
                    }
                    other => Ok(Step::Unknown(other.to_string(), v)),
                }
            }
        }

        deserializer.deserialize_map(StepVisitor)
    }
}

/// Parse a `recipe:` step value: string id or `{ id, with }`.
pub fn parse_recipe_ref(value: serde_yaml::Value) -> Result<RecipeRef, String> {
    match value {
        serde_yaml::Value::String(id) => {
            if id.trim().is_empty() {
                return Err("recipe step id must be a non-empty string".to_string());
            }
            Ok(RecipeRef {
                id,
                with: BTreeMap::new(),
            })
        }
        serde_yaml::Value::Mapping(map) => {
            let id = map
                .get(serde_yaml::Value::String("id".to_string()))
                .and_then(|v| v.as_str())
                .map(str::to_string)
                .ok_or_else(|| "recipe step mapping requires string field 'id'".to_string())?;
            if id.trim().is_empty() {
                return Err("recipe step id must be a non-empty string".to_string());
            }
            let mut with = BTreeMap::new();
            if let Some(with_val) = map.get(serde_yaml::Value::String("with".to_string())) {
                let with_map = with_val.as_mapping().ok_or_else(|| {
                    "recipe step 'with' must be a mapping of arg name to template string"
                        .to_string()
                })?;
                for (k, v) in with_map {
                    let key = k
                        .as_str()
                        .ok_or_else(|| "recipe with keys must be strings".to_string())?
                        .to_string();
                    let value = yaml_scalar_to_string(v).ok_or_else(|| {
                        format!("recipe with.{key} must be a string (or scalar) template")
                    })?;
                    with.insert(key, value);
                }
            }
            for key in map.keys() {
                let Some(name) = key.as_str() else {
                    continue;
                };
                if name != "id" && name != "with" {
                    return Err(format!(
                        "unknown field '{name}' in recipe step (expected id, with)"
                    ));
                }
            }
            Ok(RecipeRef { id, with })
        }
        _ => Err(
            "recipe step must be a recipe id string or a mapping with 'id'".to_string(),
        ),
    }
}

fn yaml_scalar_to_string(value: &serde_yaml::Value) -> Option<String> {
    match value {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Null => Some(String::new()),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateStep {
    pub path: String,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default, rename = "templateFile")]
    pub template_file: Option<String>,
    #[serde(default, rename = "ifExists")]
    pub if_exists: IfExistsStrategy,
    #[serde(default = "default_true")]
    pub format: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IfExistsStrategy {
    #[default]
    Fail,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteStep {
    pub path: String,
    #[serde(default, rename = "ifMissing")]
    pub if_missing: IfMissingStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IfMissingStrategy {
    #[default]
    Fail,
    Skip,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EditStep {
    pub path: String,
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub when: Option<GuardList>,
    #[serde(default, rename = "whenNot")]
    pub when_not: Option<GuardList>,
    #[serde(default, rename = "let")]
    pub let_bindings: LetBindings,
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
    pub query: QuerySpec,
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
    pub query: QuerySpec,
    pub capture: String,
    pub text: String,
    #[serde(default, rename = "includeLeadingTrivia")]
    pub include_leading_trivia: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct RemoveOp {
    pub query: QuerySpec,
    pub capture: String,
    #[serde(default, rename = "includeLeadingTrivia")]
    pub include_leading_trivia: bool,
}
