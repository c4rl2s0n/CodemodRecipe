//! Luau `c:component(id)` delegate: parse / verify / operation / scaffold fan-out.

mod invoke;
mod stack;

use std::rc::Rc;

use mlua::{Lua, Table, Value};

use crate::host_api::context::SharedHost;
use crate::host_api::registry::{
    rt, set_fn, HostMethodSignature, HostMethodSurface, HostReturnType,
};

use invoke::{call_child_hook, call_child_operation, call_child_scaffold};
use stack::{pop_stack, push_component};

/// Surface metadata only; runtime binding is [`register_component_delegate`].
///
/// See [`crate::host_api::registry::HOST_API_SURFACE`] for why linkme needs
/// `#[allow(unsafe_code)]` under workspace deny.
// SAFETY: linkme element expansion emits `#[unsafe(link_section)]` here.
#[allow(unsafe_code)]
#[::linkme::distributed_slice(crate::host_api::registry::HOST_API_SURFACE)]
static __HOST_API_SURFACE_COMPONENT: HostMethodSurface = HostMethodSurface {
    name: "component",
    description: "Get a component delegate (`parse` / `verify` / `readSource` / `operation` / `scaffold`) by id.",
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
    let component = crate::component::entry::with_component_host(host, |h| {
        h.components()
            .get(&id)
            .cloned()
            .ok_or_else(|| format!("unknown component '{id}'"))
    })?;

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

    // Fan-out: operations.<name>.apply(c, args, nil, graph) on the shared host.
    let host_op = Rc::clone(host);
    let component_op = component.clone();
    let id_op = id.clone();
    set_fn(
        lua,
        &delegate,
        "operation",
        move |lua, (_this, name, args): (Value, String, Option<Value>)| {
            push_component(&host_op, &id_op).map_err(rt)?;
            let result = call_child_operation(&host_op, lua, &component_op, &name, args);
            pop_stack(&host_op, &id_op);
            result.map_err(rt)
        },
    )?;

    // Fan-out: scaffold.apply(c, args, graph) on the shared host.
    let host_sc = Rc::clone(host);
    let component_sc = component.clone();
    let id_sc = id.clone();
    set_fn(
        lua,
        &delegate,
        "scaffold",
        move |lua, (_this, args): (Value, Option<Value>)| {
            push_component(&host_sc, &id_sc).map_err(rt)?;
            let result = call_child_scaffold(&host_sc, lua, &component_sc, args);
            pop_stack(&host_sc, &id_sc);
            result.map_err(rt)
        },
    )?;

    Ok(Value::Table(delegate))
}
