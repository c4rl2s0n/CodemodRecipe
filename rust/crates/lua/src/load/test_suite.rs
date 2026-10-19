use mlua::{Table, Value};

use crate::model::TestSuiteDef;

use super::error::LoadError;
use super::table_util::{optional_string, required_string};

pub fn load_test_suite_from_table(table: &Table, source: &str) -> Result<TestSuiteDef, LoadError> {
    let id = required_string(table, "id")?;
    if id.trim().is_empty() {
        return Err(LoadError::Message("test suite id is required".to_string()));
    }
    let description = optional_string(table, "description")?;
    let target = optional_string(table, "target")?;
    let tests_tbl: Table = table.get("tests").map_err(|_| {
        LoadError::Message("test suite requires tests = { name = function }".to_string())
    })?;
    let mut test_names = Vec::new();
    for pair in tests_tbl.pairs::<Value, Value>() {
        let (key, value) = pair.map_err(|e| LoadError::Message(e.to_string()))?;
        let name = match key {
            Value::String(s) => s.to_str()?.to_string(),
            other => {
                return Err(LoadError::Message(format!(
                    "test name must be a string, got {}",
                    other.type_name()
                )));
            }
        };
        if !matches!(value, Value::Function(_)) {
            return Err(LoadError::Message(format!(
                "test {name} must be a function, got {}",
                value.type_name()
            )));
        }
        test_names.push(name);
    }
    if test_names.is_empty() {
        return Err(LoadError::Message(
            "test suite requires at least one test".to_string(),
        ));
    }
    test_names.sort();
    Ok(TestSuiteDef {
        id,
        description,
        target,
        test_names,
        source: source.to_string(),
        path: None,
    })
}

