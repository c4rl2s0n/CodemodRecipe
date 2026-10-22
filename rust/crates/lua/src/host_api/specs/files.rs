use codemod_recipe_lua_macros::LuaSpec;
use mlua::{FromLua, Lua, Value};
use schemars::schema::RootSchema;
use schemars::schema_for;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::{IfExists, IfMissing};
use super::traits::{HostSpec, HostType};

/// `c:create(spec)` — create a file.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, LuaSpec)]
#[serde(rename_all = "camelCase")]
pub struct CreateSpec {
    pub path: String,
    pub content: Option<String>,
    pub template: Option<String>,
    pub if_exists: Option<IfExists>,
}

/// `c:delete(spec)` — delete a file.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, LuaSpec)]
#[serde(rename_all = "camelCase")]
pub struct DeleteSpec {
    pub path: String,
    pub if_missing: Option<IfMissing>,
}

/// `c:read(path)` — read file contents. Accepts a string path or `{ path }`.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReadSpec {
    pub path: String,
}

impl FromLua for ReadSpec {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::String(s) => Ok(Self {
                path: s.to_str()?.to_string(),
            }),
            Value::Table(t) => Ok(Self {
                path: t.get("path")?,
            }),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "ReadSpec".to_string(),
                message: Some("expected string or { path }".to_string()),
            }),
        }
    }
}

impl HostType for ReadSpec {
    const NAME: &'static str = "ReadSpec";
    fn schema() -> RootSchema {
        schema_for!(ReadSpec)
    }
}
impl HostSpec for ReadSpec {}

/// Schema shape for `c:listFiles`: one glob string or an array of globs (`!` excludes).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
enum ListFilesSchema {
    One(String),
    Many(Vec<String>),
}

/// `c:listFiles(pattern | { pattern, … })` — workspace globs; `!` prefixes exclude.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListFilesSpec {
    pub patterns: Vec<String>,
}

impl FromLua for ListFilesSpec {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::String(s) => Ok(Self {
                patterns: vec![s.to_str()?.to_string()],
            }),
            Value::Table(t) => {
                let len = t.len()?;
                let mut patterns = Vec::with_capacity(len as usize);
                for i in 1..=len {
                    let s: String = t.get(i)?;
                    patterns.push(s);
                }
                Ok(Self { patterns })
            }
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "ListFilesSpec".to_string(),
                message: Some(
                    "expected a glob string or an array of glob strings (use !pattern to exclude)"
                        .to_string(),
                ),
            }),
        }
    }
}

impl HostType for ListFilesSpec {
    const NAME: &'static str = "ListFilesSpec";
    fn schema() -> RootSchema {
        schema_for!(ListFilesSchema)
    }
}
impl HostSpec for ListFilesSpec {}
