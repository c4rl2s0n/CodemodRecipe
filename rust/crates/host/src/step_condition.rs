//! Evaluate step-level `if` / `ifNot` MiniJinja expressions.

use std::collections::BTreeMap;
use std::sync::Arc;

use minijinja::value::Value;

use crate::template::{build_condition_environment, build_template_context};

/// Returns `Ok(true)` when the condition expression is truthy (or empty/absent).
pub fn condition_expr_passes(
    if_expr: Option<&str>,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    path_exists: Arc<dyn Fn(&str) -> bool + Send + Sync>,
) -> Result<bool, String> {
    step_conditions_pass(if_expr, None, args, maps, vars, path_exists)
}

/// Returns `Ok(true)` when the step should run.
///
/// - `if` must be truthy when present
/// - `ifNot` must be falsy when present
pub fn step_conditions_pass(
    if_expr: Option<&str>,
    if_not: Option<&str>,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    path_exists: Arc<dyn Fn(&str) -> bool + Send + Sync>,
) -> Result<bool, String> {
    if let Some(expr) = if_expr {
        if !eval_condition_expr(expr, args, maps, vars, path_exists.clone())? {
            return Ok(false);
        }
    }
    if let Some(expr) = if_not {
        if eval_condition_expr(expr, args, maps, vars, path_exists)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn eval_condition_expr(
    expr: &str,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    path_exists: Arc<dyn Fn(&str) -> bool + Send + Sync>,
) -> Result<bool, String> {
    let value = eval_expression_value(expr, args, maps, vars, path_exists)?;
    Ok(value_is_truthy(&value))
}

/// Evaluate a MiniJinja expression and stringify the result (for explorerMenu args).
pub fn eval_string_expr(
    expr: &str,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    path_exists: Arc<dyn Fn(&str) -> bool + Send + Sync>,
) -> Result<String, String> {
    let value = eval_expression_value(expr, args, maps, vars, path_exists)?;
    Ok(value_to_string(&value))
}

fn eval_expression_value(
    expr: &str,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    path_exists: Arc<dyn Fn(&str) -> bool + Send + Sync>,
) -> Result<Value, String> {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Ok(Value::from(""));
    }
    let env = build_condition_environment(maps, path_exists)?;
    let compiled = env
        .compile_expression(trimmed)
        .map_err(|e| format!("invalid expression '{trimmed}': {e}"))?;
    let ctx = build_template_context(args, maps, vars);
    compiled
        .eval(ctx)
        .map_err(|e| format!("expression evaluation failed for '{trimmed}': {e}"))
}

fn value_is_truthy(value: &Value) -> bool {
    value.is_true()
}

fn value_to_string(value: &Value) -> String {
    if value.is_undefined() || value.is_none() {
        return String::new();
    }
    if let Some(s) = value.as_str() {
        return s.to_string();
    }
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checker(f: impl Fn(&str) -> bool + Send + Sync + 'static) -> Arc<dyn Fn(&str) -> bool + Send + Sync> {
        Arc::new(f)
    }

    #[test]
    fn bool_arg_gates() {
        let mut args = BTreeMap::new();
        args.insert("includeTests".to_string(), "true".to_string());
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        let exists = checker(|_p| false);
        assert!(step_conditions_pass(
            Some("includeTests"),
            None,
            &args,
            &maps,
            &vars,
            exists.clone()
        )
        .unwrap());
        args.insert("includeTests".to_string(), "false".to_string());
        assert!(!step_conditions_pass(
            Some("includeTests"),
            None,
            &args,
            &maps,
            &vars,
            exists
        )
        .unwrap());
    }

    #[test]
    fn if_not_skips_when_true() {
        let mut args = BTreeMap::new();
        args.insert("file".to_string(), "lib/a.dart".to_string());
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        assert!(!step_conditions_pass(
            None,
            Some("file | file_exists"),
            &args,
            &maps,
            &vars,
            checker(|p| p == "lib/a.dart")
        )
        .unwrap());
        assert!(step_conditions_pass(
            None,
            Some("file | file_exists"),
            &args,
            &maps,
            &vars,
            checker(|_p| false)
        )
        .unwrap());
    }

    #[test]
    fn comparison_expression() {
        let mut args = BTreeMap::new();
        args.insert("kind".to_string(), "widget".to_string());
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        let exists = checker(|_p| false);
        assert!(step_conditions_pass(
            Some("kind == \"widget\""),
            None,
            &args,
            &maps,
            &vars,
            exists.clone()
        )
        .unwrap());
        assert!(!step_conditions_pass(
            Some("kind == \"bloc\""),
            None,
            &args,
            &maps,
            &vars,
            exists
        )
        .unwrap());
    }
}
