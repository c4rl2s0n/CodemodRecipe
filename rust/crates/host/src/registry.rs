use crate::map_registry::{load_maps, merge_maps, warn_on_missing_map_ids};
use crate::protocol::{DiagnosticSource, RecipeArg, RecipeDiagnostic, RecipeSchema};
use crate::template::render_template;
use codemod_recipe_engine::engine::parse_recipe_yaml;
use codemod_recipe_yaml::compose::{expand_recipe_references, recipe_ref_id};
use codemod_recipe_yaml::model::{Arg, EditOp, Recipe, Step};
use codemod_recipe_yaml::validate::validate_recipe;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct RecipeRegistry {
    pub workspace_root: PathBuf,
    codemod_root: PathBuf,
    maps_by_id: BTreeMap<String, BTreeMap<String, String>>,
    recipes_by_id: BTreeMap<String, (PathBuf, RecipeSchema)>,
    recipes_ast: BTreeMap<String, Recipe>,
    diagnostics: Vec<RecipeDiagnostic>,
}

impl RecipeRegistry {
    pub fn new(workspace_root: PathBuf, codemod_root: PathBuf) -> Self {
        Self {
            workspace_root,
            codemod_root,
            maps_by_id: BTreeMap::new(),
            recipes_by_id: BTreeMap::new(),
            recipes_ast: BTreeMap::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn reload(&mut self) {
        self.recipes_by_id.clear();
        self.recipes_ast.clear();
        self.maps_by_id.clear();
        self.diagnostics.clear();

        let maps_dir = self.codemod_root.join("maps");
        let map_result = load_maps(&self.workspace_root, &maps_dir);
        self.maps_by_id = map_result.maps_by_id;
        self.diagnostics.extend(map_result.diagnostics);

        let recipes_dir = self.codemod_root.join("recipes");
        let Ok(entries) = std::fs::read_dir(recipes_dir) else {
            return;
        };

        let mut seen_ids: BTreeMap<String, PathBuf> = BTreeMap::new();
        let mut parsed_recipes: Vec<(PathBuf, String, Recipe)> = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "yaml" && ext != "yml") {
                continue;
            }
            let relative = relative_path(&self.workspace_root, &path);
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };

            if looks_like_map_file(&text) {
                continue;
            }

            let Ok(recipe) = parse_recipe_yaml(&text) else {
                self.diagnostics.push(RecipeDiagnostic {
                    severity: "error",
                    code: "E_RECIPE_PARSE",
                    message: format!("Failed to parse recipe: {}", path.display()),
                    sources: vec![DiagnosticSource {
                        file: relative.clone(),
                        line: None,
                        column: None,
                    }],
                });
                continue;
            };

            let schema = recipe_to_schema(&recipe);
            if seen_ids.contains_key(&schema.id) {
                self.diagnostics.push(RecipeDiagnostic {
                    severity: "error",
                    code: "E_DUPLICATE_ID",
                    message: format!("Duplicate recipe id: {}", schema.id),
                    sources: vec![DiagnosticSource {
                        file: relative,
                        line: None,
                        column: None,
                    }],
                });
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
            collect_map_warnings(recipe, relative, &self.merged_maps_for(recipe), &mut self.diagnostics);
            collect_schema_errors(recipe, relative, &mut self.diagnostics);
            collect_recipe_ref_errors(recipe, relative, &known_ids, &mut self.diagnostics);

            let schema = recipe_to_schema(recipe);
            self.recipes_ast.insert(recipe.id.clone(), recipe.clone());
            self.recipes_by_id
                .insert(schema.id.clone(), (path.clone(), schema));
        }
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

fn looks_like_map_file(text: &str) -> bool {
    let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(text) else {
        return false;
    };
    let serde_yaml::Value::Mapping(map) = value else {
        return false;
    };
    map.contains_key("entries") && !map.contains_key("steps")
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
            diagnostics.push(RecipeDiagnostic {
                severity: "error",
                code: "E_SCHEMA",
                message: "recipe step must be a recipe id string".to_string(),
                sources: vec![DiagnosticSource {
                    file: file_path.to_string(),
                    line: None,
                    column: None,
                }],
            });
            continue;
        };
        if !known_ids.contains_key(ref_id) {
            diagnostics.push(RecipeDiagnostic {
                severity: "error",
                code: "E_RECIPE_REF",
                message: format!("Unknown recipe reference: {ref_id}"),
                sources: vec![DiagnosticSource {
                    file: file_path.to_string(),
                    line: None,
                    column: None,
                }],
            });
        }
    }
}

fn collect_schema_errors(
    recipe: &Recipe,
    file_path: &str,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    if let Err(errors) = validate_recipe(recipe) {
        for error in errors {
            diagnostics.push(RecipeDiagnostic {
                severity: "error",
                code: "E_SCHEMA",
                message: error.to_string(),
                sources: vec![DiagnosticSource {
                    file: file_path.to_string(),
                    line: None,
                    column: None,
                }],
            });
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
        let Step::Edit(edit) = step else { continue };
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
        abbr: None,
        help: None,
        required: arg.required,
        defaults_to: None,
        input_kind: arg.input_kind.clone().unwrap_or_else(|| "text".to_string()),
        options: vec![],
        allow_custom_value: true,
        context_key: None,
    }
}

pub fn render_recipe_templates(
    recipe: &Recipe,
    args: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
) -> Recipe {
    let render = |text: &str| render_template(text, args, maps);
    let mut out = recipe.clone();
    for step in &mut out.steps {
        let Step::Edit(edit) = step else {
            continue;
        };
        edit.path = render(&edit.path);
        if let Some(lang) = &edit.language {
            edit.language = Some(render(lang));
        }
        for op in &mut edit.ops {
            match op {
                EditOp::Insert(insert) => {
                    insert.query = render(&insert.query);
                    insert.capture = render(&insert.capture);
                    insert.text = render(&insert.text);
                }
                EditOp::Replace(replace) => {
                    replace.query = render(&replace.query);
                    replace.capture = render(&replace.capture);
                    replace.text = render(&replace.text);
                }
                EditOp::Remove(remove) => {
                    remove.query = render(&remove.query);
                    remove.capture = render(&remove.capture);
                }
                EditOp::Unknown(_, _) => {}
            }
        }
    }
    out
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
}
