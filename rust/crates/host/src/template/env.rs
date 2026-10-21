//! MiniJinja environment construction (filters + optional `file_exists`).

use std::collections::BTreeMap;
use std::sync::Arc;

use minijinja::value::Value;
use minijinja::{Environment, UndefinedBehavior};

use crate::template::naming::{
    to_camel_case, to_kebab_case, to_lower, to_pascal_case, to_screaming_snake, to_snake_case,
    to_upper,
};
use crate::template::path_filters::{path_basename, path_parent, path_stem};

const TEMPLATE_FUEL: usize = 50_000;

/// Callback used by condition templates (`file_exists`) and explorer menu filters.
pub(crate) type PathExistsFn = Arc<dyn Fn(&str) -> bool + Send + Sync>;

pub(crate) fn build_environment(
    maps: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<Environment<'static>, String> {
    build_environment_inner(maps, None)
}

/// Environment for step `if` / `ifNot` expressions, including `file_exists`.
pub(crate) fn build_condition_environment(
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    path_exists: PathExistsFn,
) -> Result<Environment<'static>, String> {
    build_environment_inner(maps, Some(path_exists))
}

fn build_environment_inner(
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    path_exists: Option<PathExistsFn>,
) -> Result<Environment<'static>, String> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.set_fuel(Some(TEMPLATE_FUEL as u64));
    env.set_keep_trailing_newline(true);

    env.add_filter("snake_case", |value: String| -> String {
        to_snake_case(&value)
    });
    env.add_filter("camel_case", |value: String| -> String {
        to_camel_case(&value)
    });
    env.add_filter("pascal_case", |value: String| -> String {
        to_pascal_case(&value)
    });
    env.add_filter("lower", |value: String| -> String { to_lower(&value) });
    env.add_filter("upper", |value: String| -> String { to_upper(&value) });
    env.add_filter("screaming_snake", |value: String| -> String {
        to_screaming_snake(&value)
    });
    env.add_filter("kebab_case", |value: String| -> String {
        to_kebab_case(&value)
    });
    env.add_filter("trim", |value: String| -> String {
        value.trim().to_string()
    });

    env.add_filter("parent", |value: String| -> String { path_parent(&value) });
    env.add_filter("basename", |value: String| -> String {
        path_basename(&value)
    });
    env.add_filter("stem", |value: String| -> String { path_stem(&value) });

    env.add_filter("int", |value: Value| -> Result<i64, minijinja::Error> {
        value_to_i64(&value)
            .map_err(|e| minijinja::Error::new(minijinja::ErrorKind::InvalidOperation, e))
    });
    env.add_filter(
        "add",
        |value: i64, n: i64| -> Result<i64, minijinja::Error> {
            value.checked_add(n).ok_or_else(|| {
                minijinja::Error::new(
                    minijinja::ErrorKind::InvalidOperation,
                    "integer overflow in add",
                )
            })
        },
    );
    env.add_filter(
        "sub",
        |value: i64, n: i64| -> Result<i64, minijinja::Error> {
            value.checked_sub(n).ok_or_else(|| {
                minijinja::Error::new(
                    minijinja::ErrorKind::InvalidOperation,
                    "integer overflow in sub",
                )
            })
        },
    );
    env.add_filter("string", |value: i64| -> String { value.to_string() });
    env.add_filter("str", |value: i64| -> String { value.to_string() });

    let maps = Arc::new(maps.clone());
    env.add_filter(
        "map",
        move |value: String, map_id: String| -> Result<String, minijinja::Error> {
            let lookup = maps
                .get(&map_id)
                .and_then(|entries| entries.get(&value))
                .cloned()
                .unwrap_or(value);
            Ok(lookup)
        },
    );

    if let Some(path_exists) = path_exists {
        env.add_filter("file_exists", move |value: String| -> bool {
            path_exists(value.trim())
        });
    }

    Ok(env)
}

fn value_to_i64(value: &Value) -> Result<i64, String> {
    if let Some(n) = value.as_i64() {
        return Ok(n);
    }
    if let Some(s) = value.as_str() {
        let trimmed = s.trim();
        return trimmed
            .parse::<i64>()
            .map_err(|_| format!("expected integer, got '{s}'"));
    }
    Err("expected integer value".to_string())
}
