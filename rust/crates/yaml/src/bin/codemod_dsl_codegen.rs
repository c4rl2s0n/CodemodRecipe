//! Generate VS Code artifacts from the DSL structural inventory and vocabulary.
//!
//! Outputs: JSON Schema (from model-aligned structure), dsl-surface, keyword docs, TextMate.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use codemod_recipe_yaml::{
    dsl_surface_json, keyword_docs_json, map_schema, recipe_schema, syntax_alternation,
    variables_schema, SyntaxGroup,
};
use serde_json::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn vscode_root(repo: &Path) -> PathBuf {
    repo.join("vscode_extension")
}

fn write_json(path: &Path, value: &Value) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    fs::write(path, format!("{json}\n")).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_keyword_docs(out: &Path) -> Result<(), String> {
    let docs = keyword_docs_json();
    let json = serde_json::to_string_pretty(&docs).map_err(|e| e.to_string())?;
    fs::write(out, format!("{json}\n")).map_err(|e| e.to_string())?;
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
    write_json(&schemas.join("recipe.schema.json"), &recipe_schema())?;
    write_json(&schemas.join("map.schema.json"), &map_schema())?;
    write_json(&schemas.join("variables.schema.json"), &variables_schema())?;
    write_json(
        &schemas.join("generated-dsl-surface.json"),
        &dsl_surface_json(),
    )?;
    patch_tm_language_keys(&ext.join("syntaxes/codemod-recipe.tmLanguage.json"))?;

    eprintln!(
        "codemod_dsl_codegen: wrote schemas (recipe/map/variables), generated-dsl-surface.json, keyword-docs, TextMate"
    );
    Ok(())
}
