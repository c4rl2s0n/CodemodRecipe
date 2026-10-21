use codemod_recipe_lua::model::TranscriptEvent;
use codemod_recipe_lua_macros::host_method;
use mlua::{Lua, Value};
use serde_json::json;

use crate::host_api::context::SharedHost;
use crate::host_api::facets::RecipeSession;
use super::helpers::trunc_text;

#[host_method(
    name = "log",
    description = "Append values to the recipe transcript (strings, tables, numbers).",
    variadic,
    recursive = true,
    return_type = Nil,
)]
pub(crate) fn do_log(
    host: &SharedHost,
    _lua: &Lua,
    args: mlua::Variadic<Value>,
) -> Result<Value, String> {
    let (detail, json_parts) = format_log_args_rich(&args)?;
    RecipeSession::with_host(host, |mut s| {
        s.bump_ops()?;
        let expandable = !json_parts.is_empty() || detail.len() > 120;
        let payload = if json_parts.len() == 1 {
            Some(json!({ "json": json_parts[0] }))
        } else if json_parts.len() > 1 {
            Some(json!({ "json": json_parts }))
        } else if expandable {
            Some(json!({ "text": trunc_text(&detail) }))
        } else {
            None
        };
        s.transcript().push(TranscriptEvent {
            kind: "log".to_string(),
            detail,
            included_in_preview: true,
            stats: None,
            payload,
            expandable,
        });
        Ok(Value::Nil)
    })
}

fn format_log_args_rich(
    args: &mlua::Variadic<Value>,
) -> Result<(String, Vec<serde_json::Value>), String> {
    if args.is_empty() {
        return Ok((String::new(), Vec::new()));
    }
    let mut parts = Vec::with_capacity(args.len());
    let mut json_parts = Vec::new();
    for value in args.iter() {
        parts.push(format_log_value(value)?);
        if matches!(value, Value::Table(_)) {
            if let Ok(json) = codemod_recipe_lua::json_lua::lua_to_json(value) {
                json_parts.push(json);
            }
        }
    }
    Ok((parts.join(" "), json_parts))
}

fn format_log_value(value: &Value) -> Result<String, String> {
    use codemod_recipe_lua::json_lua::lua_to_json;

    match value {
        Value::String(s) => Ok(s.to_str().map_err(|e| e.to_string())?.to_string()),
        Value::Nil => Ok("null".to_string()),
        Value::Boolean(b) => Ok(b.to_string()),
        Value::Integer(i) => Ok(i.to_string()),
        Value::Number(n) => Ok(n.to_string()),
        Value::Table(_) => match lua_to_json(value) {
            Ok(json) => Ok(json.to_string()),
            Err(_) => Ok("<table>".to_string()),
        },
        Value::Function(_) => Ok("<function>".to_string()),
        Value::Thread(_) => Ok("<thread>".to_string()),
        Value::UserData(_) | Value::LightUserData(_) => Ok("<userdata>".to_string()),
        other => match lua_to_json(other) {
            Ok(json) => Ok(json.to_string()),
            Err(_) => Ok(format!("<{}>", other.type_name())),
        },
    }
}
