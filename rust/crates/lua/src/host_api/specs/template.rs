use codemod_recipe_lua_macros::LuaSpec;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::Env;

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
