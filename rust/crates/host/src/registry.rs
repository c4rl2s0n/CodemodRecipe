use crate::map_registry::{load_codemod_assets, merge_maps, warn_on_missing_map_ids};
use crate::protocol::{DiagnosticSource, RecipeArg, RecipeDiagnostic, RecipeSchema};
use crate::template::render_template;
use codemod_recipe_engine::engine::parse_recipe_yaml;
use codemod_recipe_yaml::compose::{expand_recipe_references, recipe_ref_id};
use codemod_recipe_yaml::model::{Arg, CreateStep, DeleteStep, EditOp, Recipe, Step};
use codemod_recipe_yaml::validate::validate_recipe_with;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct RecipeRegistry {
    pub workspace_root: PathBuf,
    codemod_root: PathBuf,
    pub language_config: codemod_recipe_engine::RegistryConfig,
    maps_by_id: BTreeMap<String, BTreeMap<String, String>>,
    vars_by_id: BTreeMap<String, BTreeMap<String, String>>,
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
        self.diagnostics.clear();

        let assets = load_codemod_assets(&self.workspace_root, &self.codemod_root);
        self.maps_by_id = assets.maps_by_id;
        self.vars_by_id = assets.vars_by_id;
        self.diagnostics.extend(assets.diagnostics);

        let mut seen_ids: BTreeMap<String, PathBuf> = BTreeMap::new();
        let mut parsed_recipes: Vec<(PathBuf, String, Recipe)> = Vec::new();

        for path in assets.recipe_paths {
            let relative = relative_path(&self.workspace_root, &path);
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };

            let Ok(recipe) = parse_recipe_yaml(&text) else {
                self.diagnostics.push(RecipeDiagnostic::simple(
                    "error",
                    "E_RECIPE_PARSE",
                    format!("Failed to parse recipe: {}", path.display()),
                    vec![DiagnosticSource {
                        file: relative.clone(),
                        line: None,
                        column: None,
                    }],
                ));
                continue;
            };

            let schema = recipe_to_schema(&recipe);
            if seen_ids.contains_key(&schema.id) {
                self.diagnostics.push(RecipeDiagnostic::simple(
                    "error",
                    "E_DUPLICATE_ID",
                    format!("Duplicate recipe id: {}", schema.id),
                    vec![DiagnosticSource {
                        file: relative,
                        line: None,
                        column: None,
                    }],
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

        for (path, relative, recipe) in &parsed_recipes {
            collect_reserved_arg_errors(recipe, relative, &mut self.diagnostics);
            collect_map_warnings(
                recipe,
                relative,
                &self.merged_maps_for(recipe),
                &mut self.diagnostics,
            );
            collect_schema_errors(recipe, relative, &mut self.diagnostics);
            collect_recipe_ref_errors(recipe, relative, &known_ids, &mut self.diagnostics);

            let schema = recipe_to_schema(recipe);
            self.recipes_ast.insert(recipe.id.clone(), recipe.clone());
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
        self.recipes_by_id.get(id).map(|(path, _)| {
            relative_path(&self.workspace_root, path)
        })
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
        let expanded = expand_recipe_references(&recipe, &self.recipes_ast)
            .map_err(|e| e.to_string())?;
        Ok((expanded, path.clone()))
    }

    pub fn merged_maps_for(&self, recipe: &Recipe) -> BTreeMap<String, BTreeMap<String, String>> {
        merge_maps(&self.maps_by_id, &recipe.maps)
    }

    pub fn merged_maps_for_id(&self, id: &str) -> Result<BTreeMap<String, BTreeMap<String, String>>, String> {
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
                vec![DiagnosticSource {
                    file: file_path.to_string(),
                    line: None,
                    column: None,
                }],
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
    for step in &recipe.steps {
        let Step::RecipeRef(value) = step else {
            continue;
        };
        let Some(ref_id) = recipe_ref_id(value) else {
            diagnostics.push(RecipeDiagnostic::simple(
                "error",
                "E_SCHEMA",
                "recipe step must be a recipe id string".to_string(),
                vec![DiagnosticSource {
                    file: file_path.to_string(),
                    line: None,
                    column: None,
                }],
            ));
            continue;
        };
        if !known_ids.contains_key(ref_id) {
            diagnostics.push(RecipeDiagnostic::simple(
                "error",
                "E_RECIPE_REF",
                format!("Unknown recipe reference: {ref_id}"),
                vec![DiagnosticSource {
                    file: file_path.to_string(),
                    line: None,
                    column: None,
                }],
            ));
        }
    }
}

fn collect_schema_errors(
    recipe: &Recipe,
    file_path: &str,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    if let Err(errors) = validate_recipe_with(recipe, codemod_recipe_engine::is_known_language) {
        for error in errors {
            diagnostics.push(RecipeDiagnostic::simple(
                "error",
                "E_SCHEMA",
                error.to_string(),
                vec![DiagnosticSource {
                    file: file_path.to_string(),
                    line: None,
                    column: None,
                }],
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
    for step in &recipe.steps {
        match step {
            Step::Edit(edit) => {
                warn_on_missing_map_ids(&edit.path, file_path, maps, diagnostics);
                for op in &edit.ops {
                    match op {
                        EditOp::Insert(insert) => {
                            warn_on_missing_map_ids(&insert.query, file_path, maps, diagnostics);
                            warn_on_missing_map_ids(&insert.capture, file_path, maps, diagnostics);
                            warn_on_missing_map_ids(&insert.text, file_path, maps, diagnostics);
                        }
                        EditOp::Replace(replace) => {
                            warn_on_missing_map_ids(&replace.query, file_path, maps, diagnostics);
                            warn_on_missing_map_ids(&replace.capture, file_path, maps, diagnostics);
                            warn_on_missing_map_ids(&replace.text, file_path, maps, diagnostics);
                        }
                        EditOp::Remove(remove) => {
                            warn_on_missing_map_ids(&remove.query, file_path, maps, diagnostics);
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
            Step::RecipeRef(_) | Step::Unknown(_, _) => {}
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

pub fn recipe_to_schema(recipe: &Recipe) -> RecipeSchema {
    RecipeSchema {
        id: recipe.id.clone(),
        name: recipe.name.clone().unwrap_or_else(|| recipe.id.clone()),
        description: recipe.description.clone().unwrap_or_default(),
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
    let render = |text: &str| render_template(text, args, maps, vars);
    let mut out = recipe.clone();
    for step in &mut out.steps {
        match step {
            Step::Edit(edit) => {
                edit.path = render(&edit.path)?;
                if let Some(lang) = &edit.language {
                    edit.language = Some(render(lang)?);
                }
                for op in &mut edit.ops {
                    match op {
                        EditOp::Insert(insert) => {
                            insert.query = render(&insert.query)?;
                            insert.capture = render(&insert.capture)?;
                            insert.text = render(&insert.text)?;
                        }
                        EditOp::Replace(replace) => {
                            replace.query = render(&replace.query)?;
                            replace.capture = render(&replace.capture)?;
                            replace.text = render(&replace.text)?;
                        }
                        EditOp::Remove(remove) => {
                            remove.query = render(&remove.query)?;
                            remove.capture = render(&remove.capture)?;
                        }
                        EditOp::Unknown(_, _) => {}
                    }
                }
            }
            Step::Create(CreateStep {
                path,
                template,
                template_file,
                ..
            }) => {
                *path = render(path)?;
                if let Some(text) = template {
                    *template = Some(render(text)?);
                }
                if let Some(file) = template_file {
                    *template_file = Some(render(file)?);
                }
            }
            Step::Delete(DeleteStep { path, .. }) => {
                *path = render(path)?;
            }
            Step::RecipeRef(_) | Step::Unknown(_, _) => {}
        }
    }
    Ok(out)
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
    fn warns_when_recipe_references_missing_map() {
        let workspace =
            std::env::temp_dir().join(format!("codemod_registry_map_warn_{}", std::process::id()));
        let recipes_dir = workspace.join(".codemod/recipes");
        std::fs::create_dir_all(&recipes_dir).unwrap();
        std::fs::write(
            recipes_dir.join("uses_map.yaml"),
            r#"dslVersion: 2
id: uses_map
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
        assert!(diagnostics
            .iter()
            .any(|d| d.code == "W_MAP_ID_NOT_FOUND"));

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
            r#"dslVersion: 2
id: composed
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
            r#"dslVersion: 2
id: bad_recipe
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
    fn loads_recipe_outside_recipes_directory() {
        let workspace =
            std::env::temp_dir().join(format!("codemod_registry_anydir_{}", std::process::id()));
        let nested = workspace.join(".codemod/features/foo");
        std::fs::create_dir_all(&nested).unwrap();
        std::fs::write(
            nested.join("nested_recipe.yaml"),
            r#"dslVersion: 2
id: nested_recipe
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
            r#"dslVersion: 2
id: deep_recipe
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
            registry.vars_by_id().get("deep_paths").map(|m| m.get("root")),
            Some(Some(&"lib/deep".to_string()))
        );

        let _ = std::fs::remove_dir_all(workspace);
    }
}
