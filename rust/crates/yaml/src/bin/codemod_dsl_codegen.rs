//! Generate VS Code keyword docs, schema descriptions, and syntax keyword lists from the DSL vocabulary registry.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use codemod_recipe_yaml::description_for_key;
use codemod_recipe_yaml::{keyword_docs_json, syntax_alternation, SyntaxGroup};
use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn vscode_root(repo: &Path) -> PathBuf {
    repo.join("vscode_extension")
}

fn write_keyword_docs(out: &Path) -> Result<(), String> {
    let docs = keyword_docs_json();
    let json = serde_json::to_string_pretty(&docs).map_err(|e| e.to_string())?;
    fs::write(out, format!("{json}\n")).map_err(|e| e.to_string())?;
    Ok(())
}

fn apply_descriptions_to_properties(value: &mut Value) {
    let Some(obj) = value.as_object_mut() else {
        return;
    };

    if let Some(props) = obj.get_mut("properties").and_then(Value::as_object_mut) {
        for (key, prop) in props.iter_mut() {
            if let Some(desc) = description_for_property(key, prop) {
                if let Some(prop_obj) = prop.as_object_mut() {
                    prop_obj.insert(
                        "description".to_string(),
                        Value::String(desc.to_string()),
                    );
                }
            }
            apply_descriptions_to_properties(prop);
        }
    }

    if let Some(defs) = obj.get_mut("definitions").and_then(Value::as_object_mut) {
        for def in defs.values_mut() {
            apply_descriptions_to_properties(def);
        }
    }

    for key in ["items", "oneOf", "allOf", "anyOf"] {
        if let Some(child) = obj.get_mut(key) {
            match child {
                Value::Array(arr) => {
                    for item in arr {
                        apply_descriptions_to_properties(item);
                    }
                }
                Value::Object(_) => apply_descriptions_to_properties(child),
                _ => {}
            }
        }
    }
}

/// Prefer StepKind/OpKind prose when the property is a step/op discriminator (`$ref`).
fn description_for_property(key: &str, prop: &Value) -> Option<&'static str> {
    let is_discriminator = prop
        .as_object()
        .and_then(|o| o.get("$ref"))
        .and_then(|v| v.as_str())
        .is_some_and(|r| {
            r.contains("Step")
                || r.contains("recipeRef")
                || r.contains("/insertOp")
                || r.contains("/replaceOp")
                || r.contains("/removeOp")
        });
    if is_discriminator {
        use codemod_recipe_yaml::{all_entries, VocabKind};
        if let Some(entry) = all_entries()
            .iter()
            .find(|e| e.wire == key && matches!(e.kind, VocabKind::StepKind | VocabKind::OpKind))
        {
            return Some(entry.description);
        }
    }
    description_for_key(key)
}

fn patch_schema_descriptions(schema_path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(schema_path).map_err(|e| e.to_string())?;
    let mut root: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    apply_descriptions_to_properties(&mut root);
    let out = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    fs::write(schema_path, format!("{out}\n")).map_err(|e| e.to_string())?;
    Ok(())
}

fn patch_tm_language_keys(path: &Path) -> Result<(), String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut root: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    let step = syntax_alternation(SyntaxGroup::StepKind);
    let op = syntax_alternation(SyntaxGroup::OpKind);
    let field = syntax_alternation(SyntaxGroup::FieldKey);

    let repo = root
        .get_mut("repository")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "missing repository".to_string())?;

    for (name, alt) in [
        ("step-kinds", step),
        ("op-kinds", op),
        ("field-keys", field),
    ] {
        let entry = repo
            .get_mut(name)
            .and_then(Value::as_object_mut)
            .ok_or_else(|| format!("missing repository.{name}"))?;
        entry.insert(
            "match".to_string(),
            Value::String(format!("(?<![\\w-])({alt})(?![\\w-])")),
        );
    }

    let out = serde_json::to_string_pretty(&root).map_err(|e| e.to_string())?;
    fs::write(path, format!("{out}\n")).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() -> Result<(), String> {
    let repo = repo_root();
    let ext = vscode_root(&repo);
    let schemas = ext.join("schemas");

    write_keyword_docs(&schemas.join("generated-keyword-docs.json"))?;

    for name in ["recipe.schema.json", "map.schema.json", "variables.schema.json"] {
        patch_schema_descriptions(&schemas.join(name))?;
    }

    patch_tm_language_keys(&ext.join("syntaxes/codemod-recipe.tmLanguage.json"))?;

    eprintln!(
        "codemod_dsl_codegen: wrote {}/generated-keyword-docs.json and patched schemas/syntax",
        schemas.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use codemod_recipe_yaml::all_entries;

    #[test]
    fn keyword_docs_are_unique_for_keys() {
        let mut seen = BTreeMap::new();
        for entry in all_entries() {
            if entry.parent.is_none() {
                let key = (entry.wire, entry.schema_path);
                assert!(
                    seen.insert(key, ()).is_none(),
                    "duplicate key entry: {} {:?}",
                    entry.wire,
                    entry.schema_path
                );
            }
        }
    }
}
