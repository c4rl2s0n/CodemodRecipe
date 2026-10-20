use codemod_recipe_lua_macros::LuaSpec;
use mlua::{FromLua, Lua, Value};
use schemars::schema::RootSchema;
use schemars::schema_for;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::query::QuerySpec;
use super::traits::HostType;

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
