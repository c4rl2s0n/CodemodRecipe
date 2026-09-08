use std::rc::Rc;

use codemod_recipe_lua::{eval_return_table, require_chunk_name};
use mlua::{Function, Lua, Table, Value};

use crate::host_api::context::SharedHost;
use crate::host_api::registry::{rt, set_fn, HostMethodSignature, HostMethodSurface, HostReturnType};
use crate::lua_runtime::build_ctx;

/// Surface metadata only; runtime binding is [`register_component_delegate`].
///
/// See [`crate::host_api::registry::HOST_API_SURFACE`] for why linkme needs
/// `#[allow(unsafe_code)]` under workspace deny.
// SAFETY: linkme element expansion emits `#[unsafe(link_section)]` here.
#[allow(unsafe_code)]
#[::linkme::distributed_slice(crate::host_api::registry::HOST_API_SURFACE)]
static __HOST_API_SURFACE_COMPONENT: HostMethodSurface = HostMethodSurface {
    name: "component",
    description: "Get a component delegate (`parse` / `verify` / `readSource`) by id.",
    signature: HostMethodSignature::Component,
    return_type: HostReturnType::ComponentDelegate,
    accept_string: false,
};

pub(crate) fn register_component_delegate(
    lua: &Lua,
    ctx: &Table,
    host: &SharedHost,
) -> Result<(), String> {
    let delegate_host = Rc::clone(host);
    set_fn(
        lua,
        ctx,
        "component",
        move |lua, (_this, id): (Value, String)| {
            component_delegate(&delegate_host, lua, id).map_err(rt)
        },
    )?;
    Ok(())
}

fn component_delegate(host: &SharedHost, lua: &Lua, id: String) -> Result<Value, String> {
    {
        let inner = host.lock().map_err(|e| e.to_string())?;
        if !inner.components.contains_key(&id) {
            return Err(format!("unknown component '{id}'"));
        }
    }

    let component = {
        let inner = host.lock().map_err(|e| e.to_string())?;
        inner.components.get(&id).cloned().unwrap()
    };

    let delegate = lua.create_table().map_err(|e| e.to_string())?;
    let host_parse = Rc::clone(host);
    let component_parse = component.clone();
    let id_parse = id.clone();
    set_fn(
        lua,
        &delegate,
        "parse",
        move |lua, (_this, locator): (Value, Value)| {
            push_component(&host_parse, &id_parse).map_err(rt)?;
            let result =
                call_child_hook(&host_parse, lua, &component_parse, "parse", Some(locator));
            pop_stack(&host_parse, &id_parse);
            result.map_err(rt)
        },
    )?;

    let host_verify = Rc::clone(host);
    let component_verify = component.clone();
    let id_verify = id.clone();
    set_fn(
        lua,
        &delegate,
        "verify",
        move |lua, (_this, instance): (Value, Value)| {
            push_component(&host_verify, &id_verify).map_err(rt)?;
            let result = call_child_hook(
                &host_verify,
                lua,
                &component_verify,
                "verify",
                Some(instance),
            );
            pop_stack(&host_verify, &id_verify);
            result.map_err(rt)
        },
    )?;

    let host_read = Rc::clone(host);
    let component_read = component.clone();
    let id_read = id.clone();
    set_fn(
        lua,
        &delegate,
        "readSource",
        move |lua, (_this, locator): (Value, Value)| {
            push_component(&host_read, &id_read).map_err(rt)?;
            let result = call_child_hook(
                &host_read,
                lua,
                &component_read,
                "readSource",
                Some(locator),
            );
            pop_stack(&host_read, &id_read);
            result.map_err(rt)
        },
    )?;

    Ok(Value::Table(delegate))
}

fn push_component(host: &SharedHost, id: &str) -> Result<(), String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    if inner.component_stack.iter().any(|c| c == id) {
        return Err(format!("component cycle detected: {id}"));
    }
    inner.component_stack.push(id.to_string());
    Ok(())
}

fn pop_stack(host: &SharedHost, id: &str) {
    if let Ok(mut inner) = host.lock() {
        if inner.component_stack.last().map(|s| s.as_str()) == Some(id) {
            inner.component_stack.pop();
        }
    }
}

fn call_child_hook(
    host: &SharedHost,
    lua: &Lua,
    component: &codemod_recipe_lua::model::ComponentDef,
    hook: &str,
    arg: Option<Value>,
) -> Result<Value, String> {
    let ctx_table = build_ctx(lua, Rc::clone(host))?;
    let name = component
        .path
        .as_ref()
        .map(|p| require_chunk_name(p))
        .unwrap_or_else(|| component.id.clone());
    let module = eval_return_table(lua, &component.source, &name).map_err(|e| e.to_string())?;
    let func: Function = module
        .get(hook)
        .map_err(|_| format!("component {} has no {hook} hook", component.id))?;
    match arg {
        Some(value) => func
            .call::<Value>((ctx_table, value))
            .map_err(|e| format!("component {hook} failed: {e}")),
        None => func
            .call::<Value>(ctx_table)
            .map_err(|e| format!("component {hook} failed: {e}")),
    }
}
