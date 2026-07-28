use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use codemod_recipe_yaml::compose::{expand_recipe_references, ComposeError};
use codemod_recipe_yaml::model::{CreateStep, EditOp, Recipe, Step};

use crate::diag_source::source_with_needle;
use crate::map_registry::warn_on_missing_map_ids;
use crate::protocol::{RecipeDiagnostic, ValidateResponse};
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
            recipe_id,
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
            let (code, hint) = match &err {
                ComposeError::Cycle(_) => (
                    "E_COMPOSE_CYCLE",
                    Some("Break recipe reference cycles in scaffold orchestrators".to_string()),
                ),
                ComposeError::RecipeNotFound(_) => (
                    "E_RECIPE_REF",
                    Some("Ensure the referenced recipe id exists under the codemod root".to_string()),
                ),
            };
            diagnostics.push(error(
                code,
                err.to_string(),
                file_path,
                hint,
                Some(recipe_id.to_string()),
                "recipe:",
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
                            key,
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
                    warn_on_missing_map_ids(&text, file_path, maps, diagnostics);
                    report_legacy_and_undeclared(
                        field,
                        &text,
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
            text,
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
            &var,
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

fn templated_fields_for_step(step: &Step) -> Vec<(&'static str, String)> {
    let mut out = Vec::new();
    match step {
        Step::Edit(edit) => {
            out.push(("edit.path", edit.path.clone()));
            if let Some(lang) = &edit.language {
                out.push(("edit.language", lang.clone()));
            }
            if let Some(when) = &edit.when {
                for guard in &when.guards {
                    out.push(("edit.when", guard.step_strings().join("\n")));
                }
            }
            if let Some(when_not) = &edit.when_not {
                for guard in &when_not.guards {
                    out.push(("edit.whenNot", guard.step_strings().join("\n")));
                }
            }
            for binding in &edit.let_bindings.0 {
                if let Some(query) = &binding.query {
                    out.push(("let.query", query.step_strings().join("\n")));
                }
                if let Some(capture) = &binding.capture {
                    out.push(("let.capture", capture.clone()));
                }
                if let Some(join) = &binding.join {
                    out.push(("let.join", join.clone()));
                }
                if let Some(as_tmpl) = &binding.r#as {
                    out.push(("let.as", as_tmpl.clone()));
                }
            }
            for op in &edit.ops {
                match op {
                    EditOp::Insert(insert) => {
                        out.push((
                            "insert.query",
                            insert.query.step_strings().join("\n"),
                        ));
                        out.push(("insert.capture", insert.capture.clone()));
                        out.push(("insert.text", insert.text.clone()));
                    }
                    EditOp::Replace(replace) => {
                        out.push((
                            "replace.query",
                            replace.query.step_strings().join("\n"),
                        ));
                        out.push(("replace.capture", replace.capture.clone()));
                        out.push(("replace.text", replace.text.clone()));
                    }
                    EditOp::Remove(remove) => {
                        out.push((
                            "remove.query",
                            remove.query.step_strings().join("\n"),
                        ));
                        out.push(("remove.capture", remove.capture.clone()));
                    }
                    EditOp::Unknown(_, _) => {}
                }
            }
        }
        Step::Create(create) => {
            out.push(("create.path", create.path.clone()));
            if let Some(text) = &create.template {
                out.push(("create.template", text.clone()));
            }
            if let Some(file) = &create.template_file {
                out.push(("create.templateFile", file.clone()));
            }
        }
        Step::Delete(delete) => {
            out.push(("delete.path", delete.path.clone()));
        }
        Step::RecipeRef(recipe_ref) => {
            for value in recipe_ref.with.values() {
                out.push(("recipe.with", value.clone()));
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
                template_file,
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
                    "",
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
    needle: &str,
) -> RecipeDiagnostic {
    RecipeDiagnostic {
        severity: "error",
        code,
        message,
        sources: vec![source_with_needle(file, None, needle)],
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
    needle: &str,
) -> RecipeDiagnostic {
    RecipeDiagnostic {
        severity: "warning",
        code,
        message,
        sources: vec![source_with_needle(file, None, needle)],
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
        template_file,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use codemod_recipe_yaml::model::EditStep;
    use codemod_recipe_yaml::{GuardList, LetBinding, LetBindings, QuerySpec};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static N: AtomicUsize = AtomicUsize::new(0);

    fn temp_ws(name: &str) -> std::path::PathBuf {
        let n = N.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "codemod_validate_{name}_{}_{n}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join(".codemod/recipes")).unwrap();
        dir
    }

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

    #[test]
    fn detects_undeclared_args_in_when_and_let() {
        let fields = templated_fields_for_step(&Step::Edit(EditStep {
            path: "lib/{{file}}.dart".into(),
            when: Some(GuardList {
                guards: vec![QuerySpec::single("(#eq? @x \"{{className}}\")")],
            }),
            let_bindings: LetBindings(vec![LetBinding {
                name: "n".into(),
                query: Some(QuerySpec::single("(identifier) @id (#eq? @id \"{{symbol}}\")")),
                capture: Some("{{cap}}".into()),
                r#as: Some("{{ derived }}".into()),
                ..Default::default()
            }]),
            ops: vec![],
            ..Default::default()
        }));
        let joined: String = fields
            .iter()
            .map(|(k, v)| format!("{k}:{v}"))
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("edit.when"));
        assert!(joined.contains("{{className}}"));
        assert!(joined.contains("let.query"));
        assert!(joined.contains("{{symbol}}"));
        assert!(joined.contains("let.as"));
        assert!(joined.contains("{{ derived }}"));
    }

    #[test]
    fn compose_not_found_uses_recipe_ref_code() {
        let ws = temp_ws("compose_ref");
        let recipes = ws.join(".codemod/recipes");
        std::fs::write(
            recipes.join("parent.yaml"),
            r#"id: parent
steps:
  - recipe: missing_child
"#,
        )
        .unwrap();
        let mut registry = RecipeRegistry::new(ws.clone(), ws.join(".codemod"));
        registry.reload();
        let response = validate_recipe(&registry, "parent");
        let diags = response.diagnostics.unwrap_or_default();
        assert!(
            diags.iter().any(|d| d.code == "E_RECIPE_REF"),
            "{diags:?}"
        );
        let _ = std::fs::remove_dir_all(ws);
    }

    #[test]
    fn compose_cycle_uses_compose_cycle_code() {
        let ws = temp_ws("compose_cycle");
        let recipes = ws.join(".codemod/recipes");
        std::fs::write(
            recipes.join("a.yaml"),
            r#"id: a
steps:
  - recipe: b
"#,
        )
        .unwrap();
        std::fs::write(
            recipes.join("b.yaml"),
            r#"id: b
steps:
  - recipe: a
"#,
        )
        .unwrap();
        let mut registry = RecipeRegistry::new(ws.clone(), ws.join(".codemod"));
        registry.reload();
        assert!(registry.recipes_ast().contains_key("a"));
        assert!(registry.recipes_ast().contains_key("b"));
        let mut diagnostics = Vec::new();
        let recipe = registry.recipes_ast().get("a").unwrap().clone();
        validate_expanded_recipe(
            &registry,
            "a",
            &recipe,
            ".codemod/recipes/a.yaml",
            registry.maps_by_id(),
            &mut diagnostics,
        );
        assert!(
            diagnostics.iter().any(|d| d.code == "E_COMPOSE_CYCLE"),
            "{diagnostics:?}"
        );
        let _ = std::fs::remove_dir_all(ws);
    }
}
