use std::collections::BTreeMap;

use mlua::{FromLua, Lua, Value};
use schemars::schema::RootSchema;
use schemars::schema_for;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::json_lua::lua_to_json;

use super::traits::{HostSpec, HostType};

/// Insert anchor position.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Anchor {
    Start,
    #[default]
    End,
}

impl FromLua for Anchor {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let s = String::from_lua(value, lua)?;
        match s.as_str() {
            "start" => Ok(Self::Start),
            "end" => Ok(Self::End),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: "string",
                to: "Anchor".to_string(),
                message: Some("expected 'start' or 'end'".to_string()),
            }),
        }
    }
}

impl HostType for Anchor {
    const NAME: &'static str = "Anchor";
    fn schema() -> RootSchema {
        schema_for!(Anchor)
    }
}
impl HostSpec for Anchor {}

/// What to do when creating a file that already exists.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IfExists {
    #[default]
    Fail,
    Skip,
}

impl FromLua for IfExists {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let s = String::from_lua(value, lua)?;
        match s.as_str() {
            "fail" => Ok(Self::Fail),
            "skip" => Ok(Self::Skip),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: "string",
                to: "IfExists".to_string(),
                message: Some("expected 'fail' or 'skip'".to_string()),
            }),
        }
    }
}

impl HostType for IfExists {
    const NAME: &'static str = "IfExists";
    fn schema() -> RootSchema {
        schema_for!(IfExists)
    }
}
impl HostSpec for IfExists {}

/// What to do when deleting a file that does not exist.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, JsonSchema, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IfMissing {
    #[default]
    Fail,
    Skip,
}

impl FromLua for IfMissing {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        let s = String::from_lua(value, lua)?;
        match s.as_str() {
            "fail" => Ok(Self::Fail),
            "skip" => Ok(Self::Skip),
            _ => Err(mlua::Error::FromLuaConversionError {
                from: "string",
                to: "IfMissing".to_string(),
                message: Some("expected 'fail' or 'skip'".to_string()),
            }),
        }
    }
}

impl HostType for IfMissing {
    const NAME: &'static str = "IfMissing";
    fn schema() -> RootSchema {
        schema_for!(IfMissing)
    }
}
impl HostSpec for IfMissing {}

impl From<IfExists> for codemod_recipe_core::apply::file_change::IfExists {
    fn from(value: IfExists) -> Self {
        match value {
            IfExists::Fail => Self::Fail,
            IfExists::Skip => Self::Skip,
        }
    }
}

impl From<IfMissing> for codemod_recipe_core::apply::file_change::IfMissing {
    fn from(value: IfMissing) -> Self {
        match value {
            IfMissing::Fail => Self::Fail,
            IfMissing::Skip => Self::Skip,
        }
    }
}

/// Environment table passed to template rendering. Keys are strings and values
/// are arbitrary JSON-compatible Lua values.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Env(pub BTreeMap<String, JsonValue>);

impl Env {
    pub fn into_inner(self) -> BTreeMap<String, JsonValue> {
        self.0
    }
}

impl FromLua for Env {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        let table = match value {
            Value::Table(t) => t,
            other => {
                return Err(mlua::Error::FromLuaConversionError {
                    from: other.type_name(),
                    to: "Env".to_string(),
                    message: Some("expected table".to_string()),
                })
            }
        };
        let mut out = BTreeMap::new();
        for pair in table.pairs::<String, Value>() {
            let (k, v) = pair?;
            out.insert(k, lua_to_json(&v).map_err(mlua::Error::RuntimeError)?);
        }
        Ok(Self(out))
    }
}

impl JsonSchema for Env {
    fn schema_name() -> String {
        "Env".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject::default();
        schema.instance_type = Some(schemars::schema::SingleOrVec::Single(Box::new(
            schemars::schema::InstanceType::Object,
        )));
        schema.object = Some(Box::new(schemars::schema::ObjectValidation {
            additional_properties: Some(Box::new(schemars::schema::Schema::Bool(true))),
            ..Default::default()
        }));
        schemars::schema::Schema::Object(schema)
    }
}

impl HostType for Env {
    const NAME: &'static str = "Env";
    fn schema() -> RootSchema {
        schema_for!(Env)
    }
}
impl HostSpec for Env {}
