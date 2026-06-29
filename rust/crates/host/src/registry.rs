use crate::protocol::{DiagnosticSource, RecipeArg, RecipeDiagnostic, RecipeSchema};
use crate::template::render_string;
use codemod_recipe_engine::engine::parse_recipe_yaml;
use codemod_recipe_yaml::model::{Arg, EditOp, Recipe, Step};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct RecipeRegistry {
    pub workspace_root: PathBuf,
    codemod_root: PathBuf,
    recipes_by_id: BTreeMap<String, (PathBuf, RecipeSchema)>,
    diagnostics: Vec<RecipeDiagnostic>,
}

impl RecipeRegistry {
    pub fn new(workspace_root: PathBuf, codemod_root: PathBuf) -> Self {
        Self {
            workspace_root,
            codemod_root,
            recipes_by_id: BTreeMap::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn reload(&mut self) {
        self.recipes_by_id.clear();
        self.diagnostics.clear();
        let recipes_dir = self.codemod_root.join("recipes");
        let Ok(entries) = std::fs::read_dir(recipes_dir) else {
            return;
        };

        let mut seen_ids: BTreeMap<String, PathBuf> = BTreeMap::new();

        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "yaml") {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let Ok(recipe) = parse_recipe_yaml(&text) else {
                continue;
            };
            let schema = recipe_to_schema(&recipe);
            if seen_ids.contains_key(&schema.id) {
                self.diagnostics.push(RecipeDiagnostic {
                    severity: "error",
                    code: "E_DUPLICATE_ID",
                    message: format!("Duplicate recipe id: {}", schema.id),
                    sources: vec![DiagnosticSource {
                        file: path_to_string(&path),
                        line: None,
                        column: None,
                    }],
                });
                continue;
            }
            seen_ids.insert(schema.id.clone(), path.clone());
            self.recipes_by_id.insert(schema.id.clone(), (path, schema));
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

    pub fn get(&self, id: &str) -> Option<RecipeSchema> {
        self.recipes_by_id.get(id).map(|(_, s)| s.clone())
    }

    pub fn load_recipe_ast(&self, id: &str) -> Result<(Recipe, PathBuf), String> {
        let (path, _) = self
            .recipes_by_id
            .get(id)
            .ok_or_else(|| format!("Recipe not found: {id}"))?;
        let text = std::fs::read_to_string(path).map_err(|e| format!("read failed: {e}"))?;
        let recipe = parse_recipe_yaml(&text).map_err(|e| e.to_string())?;
        Ok((recipe, path.clone()))
    }

    pub fn resolve_file_path(&self, relative: &str) -> PathBuf {
        self.workspace_root.join(relative)
    }

    pub fn codemod_root(&self) -> &Path {
        &self.codemod_root
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
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

pub fn render_recipe_templates(recipe: &Recipe, args: &BTreeMap<String, String>) -> Recipe {
    let mut out = recipe.clone();
    for step in &mut out.steps {
        let Step::Edit(edit) = step else {
            continue;
        };
        edit.path = render_string(&edit.path, args);
        if let Some(lang) = &edit.language {
            edit.language = Some(render_string(lang, args));
        }
        for op in &mut edit.ops {
            match op {
                EditOp::Insert(insert) => {
                    insert.query = render_string(&insert.query, args);
                    insert.capture = render_string(&insert.capture, args);
                    insert.text = render_string(&insert.text, args);
                }
                EditOp::Replace(replace) => {
                    replace.query = render_string(&replace.query, args);
                    replace.capture = render_string(&replace.capture, args);
                    replace.text = render_string(&replace.text, args);
                }
                EditOp::Remove(remove) => {
                    remove.query = render_string(&remove.query, args);
                    remove.capture = render_string(&remove.capture, args);
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
}
