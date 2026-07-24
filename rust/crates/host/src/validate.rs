use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use codemod_recipe_yaml::compose::expand_recipe_references;
use codemod_recipe_yaml::model::{CreateStep, EditOp, Recipe, Step};

use crate::map_registry::warn_on_missing_map_ids;
use crate::protocol::{DiagnosticSource, RecipeDiagnostic, ValidateResponse};
use crate::registry::RecipeRegistry;
use crate::template::convert_legacy_syntax;

const JINJA_KEYWORDS: &[&str] = &[
    "if", "else", "elif", "endif", "for", "endfor", "block", "endblock", "extends", "include",
    "with", "endwith", "macro", "endmacro", "call", "endcall", "filter", "endfilter", "set",
    "raw", "endraw", "true", "false", "none", "True", "False", "None", "map", "var", "LBRACE",
];

/// Reload workspace recipes/maps and run the full validation pipeline.
pub fn validate_workspace(registry: &mut RecipeRegistry) -> ValidateResponse {
    registry.reload();
    build_validate_response(registry.diagnostics())
}

/// Validate a single recipe (expanded) without reloading the registry.
pub fn validate_recipe(registry: &RecipeRegistry, recipe_id: &str) -> ValidateResponse {
    let mut diagnostics = Vec::new();
    let Some(recipe) = registry.recipes_ast().get(recipe_id) else {
        diagnostics.push(error(
            "E_RECIPE_NOT_FOUND",
            format!("Recipe not found: {recipe_id}"),
            "",
            Some(format!("Call list_recipes or check .codemod/recipes/{recipe_id}.yaml")),
            Some(recipe_id.to_string()),
        ));
        return build_validate_response(&diagnostics);
    };

    let file = registry
        .recipe_file_for(recipe_id)
        .unwrap_or_else(|| format!(".codemod/recipes/{recipe_id}.yaml"));
    let maps = registry.merged_maps_for(recipe);
    validate_expanded_recipe(
        registry,
        recipe_id,
        recipe,
        &file,
        &maps,
        &mut diagnostics,
    );
    build_validate_response(&diagnostics)
}

pub fn validate_expanded_recipes(registry: &RecipeRegistry, diagnostics: &mut Vec<RecipeDiagnostic>) {
    let ids: Vec<String> = registry.recipes_ast().keys().cloned().collect();
    for id in ids {
        let Some(recipe) = registry.recipes_ast().get(&id).cloned() else {
            continue;
        };
        let file = registry
            .recipe_file_for(&id)
            .unwrap_or_else(|| format!(".codemod/recipes/{id}.yaml"));
        let maps = registry.merged_maps_for(&recipe);
        validate_expanded_recipe(registry, &id, &recipe, &file, &maps, diagnostics);
    }
}

fn validate_expanded_recipe(
    registry: &RecipeRegistry,
    recipe_id: &str,
    recipe: &Recipe,
    file_path: &str,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    validate_recipe_with_bindings(registry, recipe, file_path, diagnostics);

    let expanded = match expand_recipe_references(recipe, registry.recipes_ast()) {
        Ok(expanded) => expanded,
        Err(err) => {
            diagnostics.push(error(
                "E_COMPOSE_CYCLE",
                err.to_string(),
                file_path,
                Some("Break recipe reference cycles in scaffold orchestrators".to_string()),
                Some(recipe_id.to_string()),
            ));
            return;
        }
    };

    let declared: BTreeSet<String> = expanded.args.iter().map(|a| a.name.clone()).collect();
    check_undeclared_args_in_steps(
        &expanded.steps,
        &declared,
        recipe_id,
        file_path,
        maps,
        diagnostics,
    );

    for step in &expanded.steps {
        visit_create_steps(step, &mut |create| {
            validate_create_step(registry, create, file_path, recipe_id, diagnostics);
        });
    }
}

fn validate_recipe_with_bindings(
    registry: &RecipeRegistry,
    recipe: &Recipe,
    file_path: &str,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    validate_with_bindings_in_steps(&recipe.steps, registry, file_path, diagnostics);
}

fn validate_with_bindings_in_steps(
    steps: &[Step],
    registry: &RecipeRegistry,
    file_path: &str,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    for step in steps {
        match step {
            Step::RecipeRef(recipe_ref) => {
                if recipe_ref.with.is_empty() {
                    continue;
                }
                let Some(child) = registry.recipes_ast().get(&recipe_ref.id) else {
                    continue;
                };
                // Direct child's declared args only — not args unioned from its children.
                let child_args: BTreeSet<String> =
                    child.args.iter().map(|a| a.name.clone()).collect();
                for key in recipe_ref.with.keys() {
                    if !child_args.contains(key) {
                        diagnostics.push(error(
                            "E_RECIPE_WITH",
                            format!(
                                "recipe '{}' with.{} does not match any argument on the referenced recipe",
                                recipe_ref.id, key
                            ),
                            file_path,
                            Some(format!(
                                "Remove with.{key} or add that arg to {}",
                                recipe_ref.id
                            )),
                            Some(recipe_ref.id.clone()),
                        ));
                    }
                }
            }
            Step::Scoped(scoped) => {
                validate_with_bindings_in_steps(
                    &scoped.steps,
                    registry,
                    file_path,
                    diagnostics,
                );
            }
            _ => {}
        }
    }
}

fn check_undeclared_args_in_steps(
    steps: &[Step],
    declared: &BTreeSet<String>,
    recipe_id: &str,
    file_path: &str,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    for step in steps {
        match step {
            Step::Scoped(scoped) => {
                for value in scoped.with.values() {
                    warn_on_missing_map_ids(value, file_path, maps, diagnostics);
                    report_legacy_and_undeclared(
                        "recipe.with",
                        value,
                        declared,
                        recipe_id,
                        file_path,
                        diagnostics,
                    );
                }
                let mut inner = declared.clone();
                inner.extend(scoped.with.keys().cloned());
                check_undeclared_args_in_steps(
                    &scoped.steps,
                    &inner,
                    recipe_id,
                    file_path,
                    maps,
                    diagnostics,
                );
            }
            _ => {
                for (field, text) in templated_fields_for_step(step) {
                    warn_on_missing_map_ids(text, file_path, maps, diagnostics);
                    report_legacy_and_undeclared(
                        field,
                        text,
                        declared,
                        recipe_id,
                        file_path,
                        diagnostics,
                    );
                }
            }
        }
    }
}

fn report_legacy_and_undeclared(
    field: &str,
    text: &str,
    declared: &BTreeSet<String>,
    recipe_id: &str,
    file_path: &str,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    if crate::template::contains_legacy_syntax(text) {
        diagnostics.push(warning(
            "W_LEGACY_TEMPLATE",
            format!(
                "Legacy template helper in {field} of recipe '{recipe_id}' — use Jinja filters (e.g. {{ arg | snake_case }})"
            ),
            file_path,
            Some("See docs/recipe-templates.md for canonical Jinja2 syntax".to_string()),
            Some(recipe_id.to_string()),
        ));
    }
    for var in extract_template_variables(text) {
        if declared.contains(&var) {
            continue;
        }
        diagnostics.push(error(
            "E_UNDECLARED_ARG",
            format!(
                "Variable '{var}' is referenced in recipe '{recipe_id}' but not declared in args"
            ),
            file_path,
            Some(format!(
                "Add `- name: {var}` to {file_path} or a referenced child recipe"
            )),
            Some(recipe_id.to_string()),
        ));
    }
}

fn visit_create_steps(step: &Step, f: &mut impl FnMut(&CreateStep)) {
    match step {
        Step::Create(create) => f(create),
        Step::Scoped(scoped) => {
            for inner in &scoped.steps {
                visit_create_steps(inner, f);
            }
        }
        _ => {}
    }
}

fn templated_fields_for_step(step: &Step) -> Vec<(&'static str, &str)> {
    let mut out = Vec::new();
    match step {
        Step::Edit(edit) => {
            out.push(("edit.path", edit.path.as_str()));
            if let Some(lang) = &edit.language {
                out.push(("edit.language", lang.as_str()));
            }
            for op in &edit.ops {
                match op {
                    EditOp::Insert(insert) => {
                        out.push(("insert.query", insert.query.as_str()));
                        out.push(("insert.capture", insert.capture.as_str()));
                        out.push(("insert.text", insert.text.as_str()));
                    }
                    EditOp::Replace(replace) => {
                        out.push(("replace.query", replace.query.as_str()));
                        out.push(("replace.capture", replace.capture.as_str()));
                        out.push(("replace.text", replace.text.as_str()));
                    }
                    EditOp::Remove(remove) => {
                        out.push(("remove.query", remove.query.as_str()));
                        out.push(("remove.capture", remove.capture.as_str()));
                    }
                    EditOp::Unknown(_, _) => {}
                }
            }
        }
        Step::Create(create) => {
            out.push(("create.path", create.path.as_str()));
            if let Some(text) = &create.template {
                out.push(("create.template", text.as_str()));
            }
            if let Some(file) = &create.template_file {
                out.push(("create.templateFile", file.as_str()));
            }
        }
        Step::Delete(delete) => {
            out.push(("delete.path", delete.path.as_str()));
        }
        Step::RecipeRef(recipe_ref) => {
            for value in recipe_ref.with.values() {
                out.push(("recipe.with", value.as_str()));
            }
        }
        Step::Scoped(_) | Step::Unknown(_, _) => {}
    }
    out
}

fn validate_create_step(
    registry: &RecipeRegistry,
    create: &CreateStep,
    file_path: &str,
    recipe_id: &str,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    if let Some(template_file) = &create.template_file {
        let path = registry.codemod_root().join(template_file);
        if !path.is_file() {
            diagnostics.push(error(
                "E_MISSING_TEMPLATE",
                format!("Template file not found: {template_file}"),
                file_path,
                Some(format!(
                    "Create {} under .codemod/ or fix templateFile path",
                    template_file
                )),
                Some(recipe_id.to_string()),
            ));
            return;
        }
        if let Ok(text) = std::fs::read_to_string(&path) {
            let converted = convert_legacy_syntax(&text);
            if let Err(err) = minijinja::Environment::new().template_from_str(&converted) {
                diagnostics.push(error(
                    "E_TEMPLATE_SYNTAX",
                    format!("Template syntax error in {template_file}: {err}"),
                    template_file,
                    None,
                    Some(recipe_id.to_string()),
                ));
            }
        }
    }
}

fn extract_template_variables(text: &str) -> BTreeSet<String> {
    let converted = convert_legacy_syntax(text);
    let mut vars = BTreeSet::new();

    let mut rest = converted.as_str();
    while let Some(start) = rest.find("{{") {
        rest = &rest[start + 2..];
        let Some(end) = rest.find("}}") else {
            break;
        };
        let inner = rest[..end].trim();
        rest = &rest[end + 2..];
        if let Some(name) = first_identifier(inner) {
            if !JINJA_KEYWORDS.contains(&name.as_str()) {
                vars.insert(name);
            }
        }
    }

    for keyword in ["if", "elif", "unless"] {
        let mut search = converted.as_str();
        while let Some(pos) = search.find(&format!("{{% {keyword} ")) {
            search = &search[pos + keyword.len() + 4..];
            if let Some(end) = search.find('%') {
                let cond = search[..end].trim();
                if let Some(name) = first_identifier(cond) {
                    if !JINJA_KEYWORDS.contains(&name.as_str()) {
                        vars.insert(name);
                    }
                }
            }
        }
    }

    vars
}

fn first_identifier(expr: &str) -> Option<String> {
    let token = expr
        .split(|c: char| c.is_whitespace() || c == '|' || c == '.')
        .find(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        })?;
    Some(token.to_string())
}

pub fn build_validate_response(diagnostics: &[RecipeDiagnostic]) -> ValidateResponse {
    let ok = diagnostics.iter().all(|d| d.severity != "error");
    ValidateResponse {
        ok,
        error: None,
        diagnostics: if diagnostics.is_empty() {
            None
        } else {
            Some(diagnostics.to_vec())
        },
    }
}

fn error(
    code: &'static str,
    message: String,
    file: &str,
    hint: Option<String>,
    related_recipe: Option<String>,
) -> RecipeDiagnostic {
    RecipeDiagnostic {
        severity: "error",
        code,
        message,
        sources: vec![DiagnosticSource {
            file: file.to_string(),
            line: None,
            column: None,
        }],
        hint,
        related_recipe,
    }
}

fn warning(
    code: &'static str,
    message: String,
    file: &str,
    hint: Option<String>,
    related_recipe: Option<String>,
) -> RecipeDiagnostic {
    RecipeDiagnostic {
        severity: "warning",
        code,
        message,
        sources: vec![DiagnosticSource {
            file: file.to_string(),
            line: None,
            column: None,
        }],
        hint,
        related_recipe,
    }
}

pub fn check_template_file_exists(
    codemod_root: &Path,
    template_file: &str,
    recipe_file: &str,
    recipe_id: &str,
) -> Option<RecipeDiagnostic> {
    let path = codemod_root.join(template_file);
    if path.is_file() {
        return None;
    }
    Some(error(
        "E_MISSING_TEMPLATE",
        format!("Template file not found: {template_file}"),
        recipe_file,
        Some(format!(
            "Create .codemod/{template_file} or update templateFile"
        )),
        Some(recipe_id.to_string()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_variables_from_template_text() {
        let vars = extract_template_variables("path: {{file}} query with {{ className }}");
        assert!(vars.contains("file"));
        assert!(vars.contains("className"));
    }

    #[test]
    fn converts_legacy_before_extract() {
        let vars = extract_template_variables("{{$camel field}}");
        assert!(vars.contains("field"));
    }
}
