//! Evaluate step-level `if` / `ifNot` MiniJinja expressions.

use std::collections::BTreeMap;
use std::sync::Arc;

use minijinja::value::Value;

use crate::template::{build_condition_environment, build_template_context};

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
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return Ok(true);
    }
    let env = build_condition_environment(maps, path_exists)?;
    let compiled = env
        .compile_expression(trimmed)
        .map_err(|e| format!("invalid step condition expression '{trimmed}': {e}"))?;
    let ctx = build_template_context(args, maps, vars);
    let value = compiled
        .eval(ctx)
        .map_err(|e| format!("step condition evaluation failed for '{trimmed}': {e}"))?;
    Ok(value_is_truthy(&value))
}

fn value_is_truthy(value: &Value) -> bool {
    value.is_true()
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
