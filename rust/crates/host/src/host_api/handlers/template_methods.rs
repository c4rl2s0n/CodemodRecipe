use codemod_recipe_lua::host_api::specs::{PostExecSpec, TemplateSpec};
use codemod_recipe_lua::model::TranscriptEvent;
use codemod_recipe_lua_macros::host_method;
use mlua::{Lua, Value};
use serde_json::json;

use crate::host_api::context::SharedHost;
use crate::template::{render_template, render_template_file};
use super::helpers::{post_exec_cmd, trunc_text};

#[host_method(
    name = "postExec",
    description = "Shell command after apply (Jinja-rendered); preview records only.",
    spec = PostExecSpec,
    return_type = Nil,
)]
pub(crate) fn do_post_exec(
    host: &SharedHost,
    _lua: &Lua,
    spec: PostExecSpec,
) -> Result<Value, String> {
    let cmd = spec
        .command
        .or(spec.path)
        .ok_or_else(|| "postExec requires command or path".to_string())?;
    post_exec_cmd(host, &cmd)?;
    Ok(Value::Nil)
}

#[host_method(
    name = "template",
    description = "Render MiniJinja from `inline` or file `path` (exactly one required).",
    spec = TemplateSpec,
    return_type = String,
)]
pub(crate) fn do_template(
    host: &SharedHost,
    lua: &Lua,
    spec: TemplateSpec,
) -> Result<Value, String> {
    let has_path = spec.path.as_ref().is_some_and(|p| !p.is_empty());
    let has_inline = spec.inline.as_ref().is_some_and(|s| !s.is_empty());
    match (has_path, has_inline) {
        (true, false) => {}
        (false, true) => {}
        (true, true) => {
            return Err("template requires exactly one of path or inline".to_string());
        }
        (false, false) => {
            return Err("template requires path or inline".to_string());
        }
    }

    let mut inner = host.lock().map_err(|e| e.to_string())?;
    let mut args = inner.args.clone();
    args.extend(spec.env.map(|e| e.into_inner()).unwrap_or_default());
    let path_label = spec.path.clone();
    let rendered = if has_path {
        render_template_file(
            spec.path.as_deref().unwrap(),
            &args,
            &inner.maps,
            &inner.vars,
            &inner.codemod_root,
            inner.recipe_path.as_deref(),
        )?
    } else {
        render_template(
            spec.inline.as_deref().unwrap(),
            &args,
            &inner.maps,
            &inner.vars,
        )?
    };
    let detail = if has_path {
        format!("path={}", path_label.as_deref().unwrap_or(""))
    } else {
        "inline".to_string()
    };
    inner.transcript.push(TranscriptEvent::with_payload(
        "template",
        detail,
        json!({
            "source": if has_path { "path" } else { "inline" },
            "path": path_label,
            "rendered": trunc_text(&rendered),
        }),
    ));
    lua.create_string(&rendered)
        .map(Value::String)
        .map_err(|e| e.to_string())
}

