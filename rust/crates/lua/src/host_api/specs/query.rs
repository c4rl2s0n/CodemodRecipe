use mlua::{FromLua, Lua, Value};
use schemars::schema::RootSchema;
use schemars::schema_for;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::traits::{HostSpec, HostType};

/// Raw query specifier: either a single string or a chain of strings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum QuerySpec {
    Single(String),
    Chain(Vec<String>),
}

impl QuerySpec {
    pub fn iter(&self) -> Box<dyn Iterator<Item = &str> + '_> {
        match self {
            QuerySpec::Single(s) => Box::new(std::iter::once(s.as_str())),
            QuerySpec::Chain(v) => Box::new(v.iter().map(String::as_str)),
        }
    }
}

impl From<QuerySpec> for codemod_recipe_core::QuerySpec {
    fn from(value: QuerySpec) -> Self {
        match value {
            QuerySpec::Single(s) => Self::Single(s),
            QuerySpec::Chain(v) => Self::Chain(v),
        }
    }
}

impl From<&QuerySpec> for codemod_recipe_core::QuerySpec {
    fn from(value: &QuerySpec) -> Self {
        match value {
            QuerySpec::Single(s) => Self::Single(s.clone()),
            QuerySpec::Chain(v) => Self::Chain(v.clone()),
        }
    }
}

impl FromLua for QuerySpec {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::String(s) => Ok(Self::Single(s.to_str()?.to_string())),
            Value::Table(t) => {
                let mut chain = Vec::new();
                for v in t.sequence_values::<String>() {
                    chain.push(v?);
                }
                Ok(Self::Chain(chain))
            }
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "QuerySpec".to_string(),
                message: Some("expected string or list of strings".to_string()),
            }),
        }
    }
}

impl HostType for QuerySpec {
    const NAME: &'static str = "QuerySpec";
    fn schema() -> RootSchema {
        schema_for!(QuerySpec)
    }
}
impl HostSpec for QuerySpec {}
