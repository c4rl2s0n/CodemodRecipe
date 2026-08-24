use mlua::{Lua, Table, Value as LuaValue};
use serde_json::{Map, Number, Value as JsonValue};

pub fn json_to_lua(lua: &Lua, value: &JsonValue) -> Result<LuaValue, String> {
    match value {
        JsonValue::Null => Ok(LuaValue::Nil),
        JsonValue::Bool(b) => Ok(LuaValue::Boolean(*b)),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(LuaValue::Integer(i))
            } else if let Some(u) = n.as_u64() {
                Ok(LuaValue::Integer(u as i64))
            } else if let Some(f) = n.as_f64() {
                Ok(LuaValue::Number(f))
            } else {
                Err("unsupported JSON number".to_string())
            }
        }
        JsonValue::String(s) => lua
            .create_string(s)
            .map(LuaValue::String)
            .map_err(|e| e.to_string()),
        JsonValue::Array(items) => {
            let tbl = lua.create_table().map_err(|e| e.to_string())?;
            for (i, item) in items.iter().enumerate() {
                tbl.set(i + 1, json_to_lua(lua, item)?)
                    .map_err(|e| e.to_string())?;
            }
            Ok(LuaValue::Table(tbl))
        }
        JsonValue::Object(map) => {
            let tbl = lua.create_table().map_err(|e| e.to_string())?;
            for (k, v) in map {
                tbl.set(k.as_str(), json_to_lua(lua, v)?)
                    .map_err(|e| e.to_string())?;
            }
            Ok(LuaValue::Table(tbl))
        }
    }
}

pub fn lua_to_json(value: &LuaValue) -> Result<JsonValue, String> {
    match value {
        LuaValue::Nil => Ok(JsonValue::Null),
        LuaValue::Boolean(b) => Ok(JsonValue::Bool(*b)),
        LuaValue::Integer(i) => Ok(JsonValue::Number((*i).into())),
        LuaValue::Number(n) => Number::from_f64(*n)
            .map(JsonValue::Number)
            .ok_or_else(|| "invalid JSON number".to_string()),
        LuaValue::String(s) => Ok(JsonValue::String(
            s.to_str().map_err(|e| e.to_string())?.to_string(),
        )),
        LuaValue::Table(tbl) => table_to_json(tbl),
        other => Err(format!(
            "cannot convert Lua {} to JSON",
            other.type_name()
        )),
    }
}

fn table_to_json(tbl: &Table) -> Result<JsonValue, String> {
    let mut max_index = 0usize;
    let mut array_len = 0usize;
    let mut object = Map::new();
    for pair in tbl.clone().pairs::<LuaValue, LuaValue>() {
        let (k, v) = pair.map_err(|e| e.to_string())?;
        match k {
            LuaValue::Integer(i) if i >= 1 => {
                array_len += 1;
                max_index = max_index.max(i as usize);
                object.insert(i.to_string(), lua_to_json(&v)?);
            }
            LuaValue::String(s) => {
                object.insert(
                    s.to_str().map_err(|e| e.to_string())?.to_string(),
                    lua_to_json(&v)?,
                );
            }
            other => {
                object.insert(format!("{:?}", other), lua_to_json(&v)?);
            }
        }
    }
    let is_array = array_len > 0 && array_len == max_index && object.len() == array_len;
    if is_array {
        let mut items = Vec::with_capacity(max_index);
        for i in 1..=max_index {
            items.push(
                object
                    .remove(&i.to_string())
                    .unwrap_or(JsonValue::Null),
            );
        }
        Ok(JsonValue::Array(items))
    } else {
        Ok(JsonValue::Object(object))
    }
}
