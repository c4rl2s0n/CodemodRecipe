use std::collections::BTreeMap;

use codemod_recipe_lua_macros::LuaSpec;
use mlua::{FromLua, Lua, Value};
use schemars::schema::RootSchema;
use schemars::schema_for;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::json_lua::lua_to_json;

/// Schema-backed type used by the host API surface generator (inputs and returns).
pub trait HostType: 'static + Send {
    /// Rust/Luau name of the type (e.g. `InsertSpec`, `QueryHit`).
    const NAME: &'static str;
    /// JSON schema used by the surface generator.
    fn schema() -> RootSchema;
}

/// Metadata trait implemented by every Luau-facing host API **input** spec type.
///
/// The `#[derive(LuaSpec)]` macro provides the implementation for simple
/// table specs; complex types (e.g. `QuerySpec`, `ReadSpec`, `Env`) implement
/// it manually. Return-only shapes implement [`HostType`] without this trait.
pub trait HostSpec: HostType + FromLua + Sized {}

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

/// `c:insert(spec)` — insert text at a tree-sitter capture.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, LuaSpec)]
#[serde(rename_all = "camelCase")]
pub struct InsertSpec {
    pub path: String,
    pub query: QuerySpec,
    pub capture: String,
    pub text: String,
    pub language: Option<String>,
    pub anchor: Option<Anchor>,
}

/// `c:replace(spec)` — replace a capture span with text.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, LuaSpec)]
#[serde(rename_all = "camelCase")]
pub struct ReplaceSpec {
    pub path: String,
    pub query: QuerySpec,
    pub capture: String,
    pub text: String,
    pub language: Option<String>,
    pub include_leading_trivia: Option<bool>,
}

/// `c:remove(spec)` — remove a capture span.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, LuaSpec)]
#[serde(rename_all = "camelCase")]
pub struct RemoveSpec {
    pub path: String,
    pub query: QuerySpec,
    pub capture: String,
    pub language: Option<String>,
    pub include_leading_trivia: Option<bool>,
}

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

/// `c:query(spec)` — requires a capture name; returns [`QueryHit`] rows.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, LuaSpec)]
#[serde(rename_all = "camelCase")]
pub struct QueryCaptureSpec {
    pub path: String,
    pub query: QuerySpec,
    pub capture: String,
    pub language: Option<String>,
}

/// Table shape for query helpers (`queryExists`, `queryCount`, `queryMatches`,
/// `expectMatch`, `expectNoMatch`) where `capture` is optional.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, LuaSpec)]
#[serde(rename_all = "camelCase")]
pub struct QueryTableSpec {
    pub path: String,
    pub query: QuerySpec,
    pub capture: Option<String>,
    pub language: Option<String>,
}

/// Byte offset span returned by query hits (`from`/`to`; Lua reserves `end`).
/// Offsets are **0-based bytes** into the file source (not Lua 1-based string indices).
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ByteSpan {
    pub from: u64,
    pub to: u64,
}

impl FromLua for ByteSpan {
    fn from_lua(value: Value, _: &Lua) -> mlua::Result<Self> {
        match value {
            Value::Table(t) => Ok(Self {
                from: t.get("from")?,
                to: t.get("to")?,
            }),
            other => Err(mlua::Error::FromLuaConversionError {
                from: other.type_name(),
                to: "ByteSpan".to_string(),
                message: Some("expected { from, to }".to_string()),
            }),
        }
    }
}

impl HostType for ByteSpan {
    const NAME: &'static str = "ByteSpan";
    fn schema() -> RootSchema {
        schema_for!(ByteSpan)
    }
}

/// `c:spanText({ path, span })` — slice file bytes by a 0-based [`ByteSpan`].
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, LuaSpec)]
#[serde(rename_all = "camelCase")]
pub struct SpanTextSpec {
    pub path: String,
    pub span: ByteSpan,
}

/// One tree-sitter query hit: span + text + node kind.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct QueryHit {
    pub span: ByteSpan,
    pub text: String,
    pub kind: String,
}

impl HostType for QueryHit {
    const NAME: &'static str = "QueryHit";
    fn schema() -> RootSchema {
        schema_for!(QueryHit)
    }
}

/// One `matchFiles` row: `path` plus `{placeholder}` captures (all string values).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchFileRow;

impl JsonSchema for MatchFileRow {
    fn schema_name() -> String {
        "MatchFileRow".to_string()
    }

    fn json_schema(_gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let mut schema = schemars::schema::SchemaObject::default();
        schema.instance_type = Some(schemars::schema::SingleOrVec::Single(Box::new(
            schemars::schema::InstanceType::Object,
        )));
        schema.object = Some(Box::new(schemars::schema::ObjectValidation {
            additional_properties: Some(Box::new(schemars::schema::Schema::Object(
                schemars::schema::SchemaObject {
                    instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(
                        schemars::schema::InstanceType::String,
                    ))),
                    ..Default::default()
                },
            ))),
            ..Default::default()
        }));
        schemars::schema::Schema::Object(schema)
    }
}

impl HostType for MatchFileRow {
    const NAME: &'static str = "MatchFileRow";
    fn schema() -> RootSchema {
        schema_for!(MatchFileRow)
    }
}

/// One `queryMatches` row: capture name → single hit or array of hits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryMatchRow;

impl JsonSchema for QueryMatchRow {
    fn schema_name() -> String {
        "QueryMatchRow".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        let hit = gen.subschema_for::<QueryHit>();
        let hit_array = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::SingleOrVec::Single(Box::new(
                schemars::schema::InstanceType::Array,
            ))),
            array: Some(Box::new(schemars::schema::ArrayValidation {
                items: Some(schemars::schema::SingleOrVec::Single(Box::new(hit.clone()))),
                ..Default::default()
            })),
            ..Default::default()
        };
        let value = schemars::schema::SchemaObject {
            subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
                any_of: Some(vec![hit, schemars::schema::Schema::Object(hit_array)]),
                ..Default::default()
            })),
            ..Default::default()
        };
        let mut schema = schemars::schema::SchemaObject::default();
        schema.instance_type = Some(schemars::schema::SingleOrVec::Single(Box::new(
            schemars::schema::InstanceType::Object,
        )));
        schema.object = Some(Box::new(schemars::schema::ObjectValidation {
            additional_properties: Some(Box::new(schemars::schema::Schema::Object(value))),
            ..Default::default()
        }));
        schemars::schema::Schema::Object(schema)
    }
}

impl HostType for QueryMatchRow {
    const NAME: &'static str = "QueryMatchRow";
    fn schema() -> RootSchema {
        schema_for!(QueryMatchRow)
    }
}

/// `c:postExec(spec)` — shell command after apply.
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, LuaSpec)]
#[serde(rename_all = "camelCase")]
pub struct PostExecSpec {
    pub command: Option<String>,
    pub path: Option<String>,
}

/// `c:template(spec)` — render MiniJinja from `inline` or file `path` (exactly one).
#[derive(Debug, Clone, PartialEq, Eq, JsonSchema, LuaSpec)]
#[serde(rename_all = "camelCase")]
pub struct TemplateSpec {
    pub path: Option<String>,
    pub inline: Option<String>,
    pub env: Option<Env>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::new_sandbox;

    fn lua() -> Lua {
        new_sandbox().expect("sandbox")
    }

    #[test]
    fn query_spec_single_string() {
        let lua = lua();
        let value = lua.load("\"foo\"").eval::<Value>().unwrap();
        let spec = QuerySpec::from_lua(value, &lua).unwrap();
        assert_eq!(spec, QuerySpec::Single("foo".to_string()));
    }

    #[test]
    fn query_spec_chain() {
        let lua = lua();
        let value = lua.load("{\"a\", \"b\"}").eval::<Value>().unwrap();
        let spec = QuerySpec::from_lua(value, &lua).unwrap();
        assert_eq!(
            spec,
            QuerySpec::Chain(vec!["a".to_string(), "b".to_string()])
        );
    }

    #[test]
    fn insert_spec_defaults() {
        let lua = lua();
        let value = lua
            .load("{path = \"p.lua\", query = \"q\", capture = \"c\", text = \"t\"}")
            .eval::<Value>()
            .unwrap();
        let spec = InsertSpec::from_lua(value, &lua).unwrap();
        assert_eq!(spec.path, "p.lua");
        assert_eq!(spec.anchor, None);
        assert_eq!(spec.language, None);
    }

    #[test]
    fn read_spec_string() {
        let lua = lua();
        let value = lua.load("\"foo.lua\"").eval::<Value>().unwrap();
        let spec = ReadSpec::from_lua(value, &lua).unwrap();
        assert_eq!(spec.path, "foo.lua");
    }

    #[test]
    fn read_spec_table() {
        let lua = lua();
        let value = lua.load("{path = \"foo.lua\"}").eval::<Value>().unwrap();
        let spec = ReadSpec::from_lua(value, &lua).unwrap();
        assert_eq!(spec.path, "foo.lua");
    }

    #[test]
    fn env_table() {
        let lua = lua();
        let value = lua.load("{a = 1, b = \"two\"}").eval::<Value>().unwrap();
        let env = Env::from_lua(value, &lua).unwrap();
        assert_eq!(env.0.get("a").unwrap(), &serde_json::json!(1));
        assert_eq!(env.0.get("b").unwrap(), &serde_json::json!("two"));
    }

    #[test]
    fn lua_spec_macro_exposes_host_spec() {
        assert_eq!(InsertSpec::NAME, "InsertSpec");
        let schema = InsertSpec::schema();
        let props = schema.schema.object.as_ref().unwrap().properties.clone();
        assert!(props.contains_key("path"));
        assert!(props.contains_key("query"));
        assert!(props.contains_key("capture"));
        assert!(props.contains_key("text"));
        assert!(props.contains_key("language"));
        assert!(props.contains_key("anchor"));
    }
}
