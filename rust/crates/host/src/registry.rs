use crate::diag_source::{source_file_only, source_with_needle};
use crate::map_registry::{load_codemod_assets, merge_maps, warn_on_missing_map_ids};
use crate::protocol::{RecipeArg, RecipeDiagnostic, RecipeSchema};
use crate::render_context::RecipeRenderContext;
use crate::template::{render_template, render_template_file};
use codemod_recipe_engine::engine::parse_recipe_yaml;
use codemod_recipe_yaml::compose::{expand_recipe_references, recipe_ref_id};
use codemod_recipe_yaml::model::{Arg, CreateStep, DeleteStep, EditOp, Recipe, RecipeRef, Step};
use codemod_recipe_yaml::query_conventions;
use codemod_recipe_yaml::validate::validate_recipe_with;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct RecipeRegistry {
    pub workspace_root: PathBuf,
    codemod_root: PathBuf,
    pub language_config: codemod_recipe_engine::RegistryConfig,
    maps_by_id: BTreeMap<String, BTreeMap<String, String>>,
    vars_by_id: BTreeMap<String, BTreeMap<String, String>>,
    queries_by_id: BTreeMap<String, BTreeMap<String, codemod_recipe_yaml::model::QueryDefinition>>,
    recipes_by_id: BTreeMap<String, (PathBuf, RecipeSchema)>,
    recipes_ast: BTreeMap<String, Recipe>,
    diagnostics: Vec<RecipeDiagnostic>,
}

impl RecipeRegistry {
    pub fn new(workspace_root: PathBuf, codemod_root: PathBuf) -> Self {
        Self {
            workspace_root,
            codemod_root,
            language_config: codemod_recipe_engine::RegistryConfig::default(),
            maps_by_id: BTreeMap::new(),
            vars_by_id: BTreeMap::new(),
            queries_by_id: BTreeMap::new(),
            recipes_by_id: BTreeMap::new(),
            recipes_ast: BTreeMap::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn reload(&mut self) {
        self.recipes_by_id.clear();
        self.recipes_ast.clear();
        self.maps_by_id.clear();
        self.vars_by_id.clear();
        self.queries_by_id.clear();
        self.diagnostics.clear();

        let assets = load_codemod_assets(&self.workspace_root, &self.codemod_root);
        self.maps_by_id = assets.maps_by_id;
        self.vars_by_id = assets.vars_by_id;
        self.queries_by_id = assets.queries_by_id;
        self.diagnostics.extend(assets.diagnostics);

        let mut seen_ids: BTreeMap<String, PathBuf> = BTreeMap::new();
        let mut parsed_recipes: Vec<(PathBuf, String, Recipe)> = Vec::new();

        for path in assets.recipe_paths {
            let relative = relative_path(&self.workspace_root, &path);
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };

            let recipe = match parse_recipe_yaml(&text) {
                Ok(recipe) => recipe,
                Err(err) => {
                    self.diagnostics.push(RecipeDiagnostic::simple(
                        "error",
                        "E_RECIPE_PARSE",
                        format!("Failed to parse recipe {}: {err}", path.display()),
                        vec![source_file_only(&relative)],
                    ));
                    continue;
                }
            };

            let schema = recipe_to_schema(&recipe, Some(&relative));
            if seen_ids.contains_key(&schema.id) {
                self.diagnostics.push(RecipeDiagnostic::simple(
                    "error",
                    "E_DUPLICATE_ID",
                    format!("Duplicate recipe id: {}", schema.id),
                    vec![source_with_needle(
                        &relative,
                        Some(&text),
                        &format!("id: {}", schema.id),
                    )],
                ));
                continue;
            }
            seen_ids.insert(schema.id.clone(), path.clone());
            parsed_recipes.push((path, relative, recipe));
        }

        let known_ids: BTreeMap<String, ()> = parsed_recipes
            .iter()
            .map(|(_, _, r)| (r.id.clone(), ()))
            .collect();

        for (_path, relative, recipe) in &parsed_recipes {
            collect_reserved_arg_errors(recipe, relative, &mut self.diagnostics);
            collect_map_warnings(
                recipe,
                relative,
                &self.merged_maps_for(recipe),
                &mut self.diagnostics,
            );
            collect_schema_errors(recipe, relative, &mut self.diagnostics);
            collect_recipe_ref_errors(recipe, relative, &known_ids, &mut self.diagnostics);
            self.recipes_ast.insert(recipe.id.clone(), recipe.clone());
        }

        for (path, relative, recipe) in &parsed_recipes {
            let schema = match expand_recipe_references(recipe, &self.recipes_ast) {
                Ok(expanded) => recipe_to_schema(&expanded, Some(relative)),
                Err(_) => recipe_to_schema(recipe, Some(relative)),
            };
            self.recipes_by_id
                .insert(schema.id.clone(), (path.clone(), schema));
        }

        let mut expanded_diagnostics = Vec::new();
        crate::validate::validate_expanded_recipes(self, &mut expanded_diagnostics);
        self.diagnostics.extend(expanded_diagnostics);
    }

    pub fn diagnostics(&self) -> &[RecipeDiagnostic] {
        &self.diagnostics
    }

    pub fn recipes_ast(&self) -> &BTreeMap<String, Recipe> {
        &self.recipes_ast
    }

    pub fn recipe_file_for(&self, id: &str) -> Option<String> {
        self.recipes_by_id
            .get(id)
            .map(|(path, _)| relative_path(&self.workspace_root, path))
    }

    pub fn list(&self) -> (Vec<RecipeSchema>, Vec<RecipeDiagnostic>) {
        let recipes = self
            .recipes_by_id
            .values()
            .map(|(_, s)| s.clone())
            .collect();
        (recipes, self.diagnostics.clone())
    }

    pub fn list_ids(&self) -> Vec<String> {
        self.recipes_by_id.keys().cloned().collect()
    }

    pub fn maps_count(&self) -> usize {
        self.maps_by_id.len()
    }

    pub fn vars_by_id(&self) -> &BTreeMap<String, BTreeMap<String, String>> {
        &self.vars_by_id
    }

    pub fn maps_by_id(&self) -> &BTreeMap<String, BTreeMap<String, String>> {
        &self.maps_by_id
    }

    pub fn queries_by_id(
        &self,
    ) -> &BTreeMap<String, BTreeMap<String, codemod_recipe_yaml::model::QueryDefinition>> {
        &self.queries_by_id
    }

    pub fn map_ids(&self) -> Vec<String> {
        self.maps_by_id.keys().cloned().collect()
    }

    pub fn var_ids(&self) -> Vec<String> {
        self.vars_by_id.keys().cloned().collect()
    }

    pub fn get(&self, id: &str) -> Option<RecipeSchema> {
        self.recipes_by_id.get(id).map(|(_, s)| s.clone())
    }

    pub fn load_recipe_ast(&self, id: &str) -> Result<(Recipe, PathBuf), String> {
        let (path, _) = self
            .recipes_by_id
            .get(id)
            .ok_or_else(|| format!("Recipe not found: {id}"))?;
        let recipe = self
            .recipes_ast
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Recipe AST not cached: {id}"))?;
        let expanded =
            expand_recipe_references(&recipe, &self.recipes_ast).map_err(|e| e.to_string())?;
        Ok((expanded, path.clone()))
    }

    pub fn merged_maps_for(&self, recipe: &Recipe) -> BTreeMap<String, BTreeMap<String, String>> {
        merge_maps(&self.maps_by_id, &recipe.maps)
    }

    pub fn merged_maps_for_id(
        &self,
        id: &str,
    ) -> Result<BTreeMap<String, BTreeMap<String, String>>, String> {
        let (recipe, _) = self.load_recipe_ast(id)?;
        Ok(self.merged_maps_for(&recipe))
    }

    pub fn resolve_file_path(&self, relative: &str) -> PathBuf {
        self.workspace_root.join(relative)
    }

    pub fn codemod_root(&self) -> &Path {
        &self.codemod_root
    }
}

fn collect_reserved_arg_errors(
    recipe: &Recipe,
    file_path: &str,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    for arg in &recipe.args {
        if arg.name == "map" || arg.name == "var" {
            diagnostics.push(RecipeDiagnostic::simple(
                "error",
                "E_RESERVED_ARG",
                format!(
                    "Argument name '{}' shadows reserved template namespace",
                    arg.name
                ),
                vec![source_with_needle(
                    file_path,
                    None,
                    &format!("name: {}", arg.name),
                )],
            ));
        }
    }
}

fn collect_recipe_ref_errors(
    recipe: &Recipe,
    file_path: &str,
    known_ids: &BTreeMap<String, ()>,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    collect_recipe_ref_errors_in_steps(&recipe.steps, file_path, known_ids, diagnostics);
}

fn collect_recipe_ref_errors_in_steps(
    steps: &[Step],
    file_path: &str,
    known_ids: &BTreeMap<String, ()>,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    for step in steps {
        match step {
            Step::RecipeRef(recipe_ref) => {
                validate_one_recipe_ref(recipe_ref, file_path, known_ids, diagnostics);
            }
            Step::Scoped(scoped) => {
                collect_recipe_ref_errors_in_steps(
                    &scoped.steps,
                    file_path,
                    known_ids,
                    diagnostics,
                );
            }
            _ => {}
        }
    }
}

fn validate_one_recipe_ref(
    recipe_ref: &RecipeRef,
    file_path: &str,
    known_ids: &BTreeMap<String, ()>,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    let ref_id = recipe_ref_id(recipe_ref);
    if ref_id.trim().is_empty() {
        diagnostics.push(RecipeDiagnostic::simple(
            "error",
            "E_SCHEMA",
            "recipe step must have a non-empty id".to_string(),
            vec![source_with_needle(file_path, None, "recipe:")],
        ));
        return;
    }
    if !known_ids.contains_key(ref_id) {
        diagnostics.push(RecipeDiagnostic::simple(
            "error",
            "E_RECIPE_REF",
            format!("Unknown recipe reference: {ref_id}"),
            vec![source_with_needle(file_path, None, ref_id)],
        ));
    }
}

fn collect_schema_errors(
    recipe: &Recipe,
    file_path: &str,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    if let Err(errors) = validate_recipe_with(recipe, codemod_recipe_engine::is_known_language) {
        for error in errors {
            let needle = error.to_string();
            diagnostics.push(RecipeDiagnostic::simple(
                "error",
                "E_SCHEMA",
                needle.clone(),
                vec![source_file_only(file_path)],
            ));
        }
    }
}

fn collect_map_warnings(
    recipe: &Recipe,
    file_path: &str,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    collect_map_warnings_in_steps(&recipe.steps, file_path, maps, diagnostics);
}

fn collect_map_warnings_in_steps(
    steps: &[Step],
    file_path: &str,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    for step in steps {
        match step {
            Step::Edit(edit) => {
                warn_on_missing_map_ids(&edit.path, file_path, maps, diagnostics);
                for op in &edit.ops {
                    match op {
                        EditOp::Insert(insert) => {
                            for q in insert.query.step_strings() {
                                warn_on_missing_map_ids(q, file_path, maps, diagnostics);
                            }
                            warn_on_missing_map_ids(&insert.capture, file_path, maps, diagnostics);
                            warn_on_missing_map_ids(&insert.text, file_path, maps, diagnostics);
                        }
                        EditOp::Replace(replace) => {
                            for q in replace.query.step_strings() {
                                warn_on_missing_map_ids(q, file_path, maps, diagnostics);
                            }
                            warn_on_missing_map_ids(&replace.capture, file_path, maps, diagnostics);
                            warn_on_missing_map_ids(&replace.text, file_path, maps, diagnostics);
                        }
                        EditOp::Remove(remove) => {
                            for q in remove.query.step_strings() {
                                warn_on_missing_map_ids(q, file_path, maps, diagnostics);
                            }
                            warn_on_missing_map_ids(&remove.capture, file_path, maps, diagnostics);
                        }
                        EditOp::Unknown(_, _) => {}
                    }
                }
            }
            Step::Create(create) => {
                warn_on_missing_map_ids(&create.path, file_path, maps, diagnostics);
                if let Some(text) = &create.template {
                    warn_on_missing_map_ids(text, file_path, maps, diagnostics);
                }
                if let Some(file) = &create.template_file {
                    warn_on_missing_map_ids(file, file_path, maps, diagnostics);
                }
            }
            Step::Delete(delete) => {
                warn_on_missing_map_ids(&delete.path, file_path, maps, diagnostics);
            }
            Step::RecipeRef(recipe_ref) => {
                for value in recipe_ref.with.values() {
                    warn_on_missing_map_ids(value, file_path, maps, diagnostics);
                }
            }
            Step::Scoped(scoped) => {
                for value in scoped.with.values() {
                    warn_on_missing_map_ids(value, file_path, maps, diagnostics);
                }
                collect_map_warnings_in_steps(&scoped.steps, file_path, maps, diagnostics);
            }
            Step::Unknown(_, _) => {}
        }
    }
}

fn relative_path(workspace_root: &Path, absolute: &Path) -> String {
    let root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());
    let file = absolute
        .canonicalize()
        .unwrap_or_else(|_| absolute.to_path_buf());
    if let Ok(rel) = file.strip_prefix(&root) {
        rel.to_string_lossy().to_string()
    } else {
        absolute.to_string_lossy().to_string()
    }
}

pub fn recipe_to_schema(recipe: &Recipe, source_file: Option<&str>) -> RecipeSchema {
    RecipeSchema {
        id: recipe.id.clone(),
        name: recipe.name.clone().unwrap_or_else(|| recipe.id.clone()),
        description: recipe.description.clone().unwrap_or_default(),
        source_file: source_file.map(str::to_string),
        args: recipe.args.iter().map(arg_to_schema).collect(),
    }
}

fn arg_to_schema(arg: &Arg) -> RecipeArg {
    RecipeArg {
        name: arg.name.clone(),
        abbr: arg.abbr.clone(),
        help: arg.help.clone(),
        required: arg.required,
        defaults_to: arg.defaults_to.clone(),
        input_kind: arg.input_kind.clone().unwrap_or_else(|| "text".to_string()),
        options: arg.options.clone(),
        allow_custom_value: arg.allow_custom_value.unwrap_or(true),
        context_key: arg.context_key.clone(),
    }
}

pub fn render_recipe_templates(
    recipe: &Recipe,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<Recipe, String> {
    render_recipe_templates_with_root(recipe, args, maps, vars, None, None, None)
}

/// Like [`render_recipe_templates`], and when `codemod_root` is set, materializes
/// `create.templateFile` into `create.template` using the active arg overlay
/// (needed for scoped `with` bindings).
pub fn render_recipe_templates_with_root(
    recipe: &Recipe,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    codemod_root: Option<&Path>,
    registry: Option<&RecipeRegistry>,
    recipe_file: Option<&Path>,
) -> Result<Recipe, String> {
    let mut out = recipe.clone();
    out.steps = render_steps(
        &recipe.steps,
        recipe,
        args,
        maps,
        vars,
        codemod_root,
        registry,
        recipe_file,
    )?;
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn render_steps(
    steps: &[Step],
    recipe: &Recipe,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    codemod_root: Option<&Path>,
    registry: Option<&RecipeRegistry>,
    recipe_file: Option<&Path>,
) -> Result<Vec<Step>, String> {
    let mut out = Vec::new();
    for step in steps {
        match step {
            Step::Scoped(scoped) => {
                let local = apply_with_overlay(args, &scoped.with, maps, vars)?;
                out.extend(render_steps(
                    &scoped.steps,
                    recipe,
                    &local,
                    maps,
                    vars,
                    codemod_root,
                    registry,
                    recipe_file,
                )?);
            }
            Step::Edit(edit) => {
                out.push(Step::Edit(render_edit(
                    edit,
                    recipe,
                    args,
                    maps,
                    vars,
                    codemod_root,
                    registry,
                    recipe_file,
                )?));
            }
            Step::Create(create) => {
                out.push(Step::Create(render_create(
                    create,
                    args,
                    maps,
                    vars,
                    codemod_root,
                )?));
            }
            Step::Delete(delete) => {
                out.push(Step::Delete(DeleteStep {
                    path: render_template(&delete.path, args, maps, vars)?,
                    if_missing: delete.if_missing.clone(),
                }));
            }
            Step::RecipeRef(_) | Step::Unknown(_, _) => {}
        }
    }
    Ok(out)
}

fn apply_with_overlay(
    parent_args: &BTreeMap<String, String>,
    with: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
) -> Result<BTreeMap<String, String>, String> {
    let mut local = parent_args.clone();
    for (key, template) in with {
        let value = render_template(template, parent_args, maps, vars)?;
        local.insert(key.clone(), value);
    }
    Ok(local)
}

#[allow(clippy::too_many_arguments)]
fn render_edit(
    edit: &codemod_recipe_yaml::model::EditStep,
    recipe: &Recipe,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    codemod_root: Option<&Path>,
    registry: Option<&RecipeRegistry>,
    recipe_file: Option<&Path>,
) -> Result<codemod_recipe_yaml::model::EditStep, String> {
    let render = RecipeRenderContext {
        recipe,
        registry,
        recipe_file,
        codemod_root,
        args,
        maps,
        vars,
    };
    let mut edit = edit.clone();
    edit.path = render_template(&edit.path, args, maps, vars)?;
    if let Some(lang) = &edit.language {
        edit.language = Some(render_template(lang, args, maps, vars)?);
    }
    if let Some(when) = &mut edit.when {
        render_guard_list(when, &render)?;
    }
    if let Some(when_not) = &mut edit.when_not {
        render_guard_list(when_not, &render)?;
    }
    let defer_op_render =
        !edit.let_bindings.0.is_empty() || edit.when.is_some() || edit.when_not.is_some();
    if !defer_op_render {
        for op in &mut edit.ops {
            match op {
                EditOp::Insert(insert) => {
                    insert.query = render_query_op(&insert.query, &render, args)?;
                    insert.capture = render_template(&insert.capture, args, maps, vars)?;
                    insert.text = render_template(&insert.text, args, maps, vars)?;
                }
                EditOp::Replace(replace) => {
                    replace.query = render_query_op(&replace.query, &render, args)?;
                    replace.capture = render_template(&replace.capture, args, maps, vars)?;
                    replace.text = render_template(&replace.text, args, maps, vars)?;
                }
                EditOp::Remove(remove) => {
                    remove.query = render_query_op(&remove.query, &render, args)?;
                    remove.capture = render_template(&remove.capture, args, maps, vars)?;
                }
                EditOp::Unknown(_, _) => {}
            }
        }
    }
    Ok(edit)
}

fn render_guard_list(
    list: &mut codemod_recipe_yaml::GuardList,
    render: &RecipeRenderContext<'_>,
) -> Result<(), String> {
    for spec in &mut list.guards {
        *spec = render_query_op(spec, render, render.args)?;
    }
    Ok(())
}

/// Public wrapper for query rendering (guards / per-op let).
pub fn render_query_op_public(
    spec: &codemod_recipe_yaml::model::QuerySpec,
    render: &RecipeRenderContext<'_>,
    args: &BTreeMap<String, String>,
) -> Result<codemod_recipe_yaml::model::QuerySpec, String> {
    render_query_op(spec, render, args)
}

fn render_query_op(
    spec: &codemod_recipe_yaml::model::QuerySpec,
    render: &RecipeRenderContext<'_>,
    args: &BTreeMap<String, String>,
) -> Result<codemod_recipe_yaml::model::QuerySpec, String> {
    if let Some(reg) = render.registry {
        return crate::query_resolver::render_query_spec(
            spec,
            render.recipe,
            reg,
            render.recipe_file,
            args,
            render.maps,
            render.vars,
        );
    }
    // No registry: Jinja-render inline/path steps only.
    let steps: Vec<String> = match spec {
        codemod_recipe_yaml::model::QuerySpec::Single(s) => vec![s.clone()],
        codemod_recipe_yaml::model::QuerySpec::Chain(v) => v.clone(),
    };
    let mut out = Vec::new();
    for step in steps {
        let loaded = if let Some(root) = render.codemod_root {
            if query_conventions::looks_like_query_path(&step) {
                codemod_recipe_engine::query::resolve_query_source(&step, render.recipe_file, root)
                    .map_err(|e| e.to_string())?
            } else {
                step
            }
        } else {
            step
        };
        out.push(render_template(&loaded, args, render.maps, render.vars)?);
    }
    Ok(if out.len() == 1 {
        codemod_recipe_yaml::model::QuerySpec::Single(out.into_iter().next().unwrap())
    } else {
        codemod_recipe_yaml::model::QuerySpec::Chain(out)
    })
}

fn render_create(
    create: &CreateStep,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    codemod_root: Option<&Path>,
) -> Result<CreateStep, String> {
    let mut create = create.clone();
    create.path = render_template(&create.path, args, maps, vars)?;
    if let Some(text) = &create.template {
        create.template = Some(render_template(text, args, maps, vars)?);
    }
    if let Some(file) = create.template_file.clone() {
        let rendered_name = render_template(&file, args, maps, vars)?;
        if let Some(root) = codemod_root {
            let content = render_template_file(&rendered_name, args, maps, vars, root)?;
            create.template = Some(content);
            create.template_file = None;
        } else {
            create.template_file = Some(rendered_name);
        }
    }
    Ok(create)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
    }

    #[test]
    fn loads_insert_log_line_from_repo_fixture() {
        let repo_root = repo_root();
        let codemod_root = repo_root.join(".codemod");
        let mut registry = RecipeRegistry::new(repo_root.clone(), codemod_root);
        registry.reload();

        let schema = registry
            .get("insert_log_line")
            .expect("insert_log_line recipe should load");
        assert_eq!(schema.id, "insert_log_line");
        assert!(schema.args.iter().any(|a| a.name == "file"));
        assert!(registry.maps_count() >= 1);
    }

    #[test]
    fn reports_duplicate_recipe_ids() {
        let workspace =
            std::env::temp_dir().join(format!("codemod_registry_dup_{}", std::process::id()));
        let recipes_dir = workspace.join(".codemod/recipes");
        std::fs::create_dir_all(&recipes_dir).unwrap();

        let oracle = repo_root().join("test/fixtures/rust_oracle");
        std::fs::copy(
            oracle.join("insert_log_line.recipe.yaml"),
            recipes_dir.join("insert_log_line.yaml"),
        )
        .unwrap();
        std::fs::copy(
            oracle.join("duplicate_insert_log_line.recipe.yaml"),
            recipes_dir.join("duplicate_insert_log_line.yaml"),
        )
        .unwrap();

        let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
        registry.reload();

        assert!(registry.get("insert_log_line").is_some());
        let (_, diagnostics) = registry.list();
        assert!(diagnostics.iter().any(|d| d.code == "E_DUPLICATE_ID"));

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn allows_same_leaf_id_under_different_dotted_prefixes() {
        let workspace = std::env::temp_dir().join(format!(
            "codemod_registry_dotted_ids_{}",
            std::process::id()
        ));
        let recipes_dir = workspace.join(".codemod/recipes");
        std::fs::create_dir_all(&recipes_dir).unwrap();
        std::fs::write(
            recipes_dir.join("rust_insert.yaml"),
            r#"id: rust.logging.insert
steps:
  - delete:
      path: rust.log
      ifMissing: skip
"#,
        )
        .unwrap();
        std::fs::write(
            recipes_dir.join("dart_insert.yaml"),
            r#"id: dart.logging.insert
steps:
  - delete:
      path: dart.log
      ifMissing: skip
"#,
        )
        .unwrap();

        let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
        registry.reload();

        let ids = registry.list_ids();
        assert!(ids.contains(&"rust.logging.insert".to_string()));
        assert!(ids.contains(&"dart.logging.insert".to_string()));
        let (_, diagnostics) = registry.list();
        assert!(
            diagnostics.iter().all(|d| d.code != "E_DUPLICATE_ID"),
            "unexpected duplicate diagnostics: {diagnostics:?}"
        );

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn expands_full_dotted_recipe_references() {
        let workspace = std::env::temp_dir().join(format!(
            "codemod_registry_dotted_refs_{}",
            std::process::id()
        ));
        let recipes_dir = workspace.join(".codemod/recipes");
        std::fs::create_dir_all(&recipes_dir).unwrap();
        std::fs::write(
            recipes_dir.join("child.yaml"),
            r#"id: shared.logging.insert
args:
  - name: file
    required: true
steps:
  - delete:
      path: "{{file}}"
      ifMissing: skip
"#,
        )
        .unwrap();
        std::fs::write(
            recipes_dir.join("parent.yaml"),
            r#"id: app.logging.apply
steps:
  - recipe: shared.logging.insert
"#,
        )
        .unwrap();

        let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
        registry.reload();

        let (recipe, _) = registry.load_recipe_ast("app.logging.apply").unwrap();
        assert_eq!(recipe.steps.len(), 1);
        assert!(recipe.args.iter().any(|arg| arg.name == "file"));

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn warns_when_recipe_references_missing_map() {
        let workspace =
            std::env::temp_dir().join(format!("codemod_registry_map_warn_{}", std::process::id()));
        let recipes_dir = workspace.join(".codemod/recipes");
        std::fs::create_dir_all(&recipes_dir).unwrap();
        std::fs::write(
            recipes_dir.join("uses_map.yaml"),
            r#"id: uses_map
args:
  - name: file
    required: true
steps:
  - edit:
      path: "{{file}}"
      ops:
        - insert:
            query: "(identifier) @x"
            capture: x
            anchor: start
            text: "{{$map 'missing_map' type}}"
"#,
        )
        .unwrap();

        let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
        registry.reload();

        let (_, diagnostics) = registry.list();
        assert!(diagnostics.iter().any(|d| d.code == "W_MAP_ID_NOT_FOUND"));

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn expands_recipe_references_on_load() {
        let workspace =
            std::env::temp_dir().join(format!("codemod_registry_compose_{}", std::process::id()));
        let recipes_dir = workspace.join(".codemod/recipes");
        std::fs::create_dir_all(&recipes_dir).unwrap();

        let oracle = repo_root().join("test/fixtures/rust_oracle");
        std::fs::copy(
            oracle.join("add_counter_field.recipe.yaml"),
            recipes_dir.join("add_counter_field.yaml"),
        )
        .unwrap();
        std::fs::copy(
            oracle.join("add_log_line.recipe.yaml"),
            recipes_dir.join("add_log_line.yaml"),
        )
        .unwrap();
        std::fs::write(
            recipes_dir.join("composed.yaml"),
            r#"id: composed
args:
  - name: file
    required: true
  - name: className
    required: true
  - name: field
    required: true
  - name: methodName
    required: true
steps:
  - recipe: add_counter_field
  - recipe: add_log_line
"#,
        )
        .unwrap();

        let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
        registry.reload();

        let (recipe, _) = registry.load_recipe_ast("composed").unwrap();
        assert_eq!(recipe.steps.len(), 2);
        assert!(recipe.args.iter().any(|a| a.name == "file"));
        assert!(recipe.args.iter().any(|a| a.name == "field"));
        assert!(recipe.args.iter().any(|a| a.name == "methodName"));

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn reports_schema_errors_for_invalid_recipe() {
        let workspace =
            std::env::temp_dir().join(format!("codemod_registry_schema_{}", std::process::id()));
        let recipes_dir = workspace.join(".codemod/recipes");
        std::fs::create_dir_all(&recipes_dir).unwrap();
        std::fs::write(
            recipes_dir.join("bad.yaml"),
            r#"id: bad_recipe
steps:
  - edit:
      path: "a.dart"
      ops: []
"#,
        )
        .unwrap();

        let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
        registry.reload();

        let (_, diagnostics) = registry.list();
        assert!(diagnostics.iter().any(|d| d.code == "E_SCHEMA"));

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn rejects_legacy_group_field() {
        let workspace = std::env::temp_dir().join(format!(
            "codemod_registry_group_field_{}",
            std::process::id()
        ));
        let recipes_dir = workspace.join(".codemod/recipes");
        std::fs::create_dir_all(&recipes_dir).unwrap();
        std::fs::write(
            recipes_dir.join("bad.yaml"),
            r#"id: demo.recipe
group: legacy.examples
steps:
  - delete:
      path: "demo.txt"
      ifMissing: skip
"#,
        )
        .unwrap();

        let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
        registry.reload();

        assert!(registry.get("demo.recipe").is_none());
        let (_, diagnostics) = registry.list();
        assert!(diagnostics.iter().any(|d| {
            d.code == "E_RECIPE_PARSE"
                && d.message
                    .contains("top-level field 'group' is no longer supported")
        }));

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn reports_unknown_with_keys_on_recipe_ref() {
        let workspace =
            std::env::temp_dir().join(format!("codemod_registry_bad_with_{}", std::process::id()));
        let recipes_dir = workspace.join(".codemod/recipes");
        std::fs::create_dir_all(&recipes_dir).unwrap();
        std::fs::write(
            recipes_dir.join("child.yaml"),
            r#"id: child
args:
  - name: className
    required: true
steps:
  - create:
      path: "lib/{{ className }}.dart"
      template: "class {{ className }} {}"
      ifExists: fail
"#,
        )
        .unwrap();
        std::fs::write(
            recipes_dir.join("parent.yaml"),
            r#"id: parent
args:
  - name: featureName
    required: true
steps:
  - recipe:
      id: child
      with:
        className: "{{ featureName }}"
        notARealArg: "x"
"#,
        )
        .unwrap();

        let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
        registry.reload();

        let (_, diagnostics) = registry.list();
        assert!(
            diagnostics
                .iter()
                .any(|d| d.code == "E_RECIPE_WITH" && d.message.contains("notARealArg")),
            "expected E_RECIPE_WITH for unknown with key, got: {diagnostics:?}"
        );

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn loads_recipe_outside_recipes_directory() {
        let workspace =
            std::env::temp_dir().join(format!("codemod_registry_anydir_{}", std::process::id()));
        let nested = workspace.join(".codemod/features/foo");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(
            nested.join("nested_recipe.yaml"),
            r#"id: nested_recipe
args:
  - name: file
    required: true
steps:
  - delete:
      path: "{{file}}"
      ifMissing: skip
"#,
        )
        .unwrap();

        let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
        registry.reload();
        assert!(registry.get("nested_recipe").is_some());

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn loads_recipe_map_and_var_from_nested_tree() {
        let workspace = std::env::temp_dir().join(format!(
            "codemod_registry_nested_tree_{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&workspace);
        let nested = workspace.join(".codemod/a/b/c");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(
            nested.join("deep_recipe.yaml"),
            r#"id: deep_recipe
args:
  - name: file
    required: true
steps:
  - delete:
      path: "{{file}}"
      ifMissing: skip
"#,
        )
        .unwrap();
        std::fs::write(
            nested.join("deep_map.yaml"),
            "id: deep_types\nmap:\n  x: int\n",
        )
        .unwrap();
        std::fs::write(
            nested.join("deep_vars.yaml"),
            "id: deep_paths\nvalues:\n  root: lib/deep\n",
        )
        .unwrap();

        let mut registry = RecipeRegistry::new(workspace.clone(), workspace.join(".codemod"));
        registry.reload();

        let (_, diagnostics) = registry.list();
        assert!(
            diagnostics.iter().all(|d| d.severity != "error"),
            "diagnostics: {diagnostics:?}"
        );
        assert!(registry.get("deep_recipe").is_some());
        assert!(registry.maps_count() >= 1);
        assert_eq!(
            registry
                .vars_by_id()
                .get("deep_paths")
                .map(|m| m.get("root")),
            Some(Some(&"lib/deep".to_string()))
        );

        let _ = std::fs::remove_dir_all(workspace);
    }
}
