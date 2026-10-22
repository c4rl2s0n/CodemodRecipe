use codemod_recipe_lua_macros::LuaSpec;
use schemars::JsonSchema;

use super::common::Anchor;
use super::query::QuerySpec;

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
