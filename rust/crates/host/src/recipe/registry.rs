use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use codemod_recipe_lua::load::{load_map, load_query_lib, load_recipe, load_variables, LoadError};
use codemod_recipe_lua::model::{Arg, ArgFrom, ExplorerMenuEntry, Recipe};

use crate::diagnostics::diag_source::source_file_only;
use crate::protocol::{ExplorerMenuEntrySchema, RecipeArg, RecipeDiagnostic, RecipeSchema};

pub struct RecipeRegistry {
    pub workspace_root: PathBuf,
    codemod_root: PathBuf,
    pub language_registry: codemod_recipe_engine::LanguageRegistry,
    pub ast_cache: crate::query::ast_cache::AstParseCache,
    maps_by_id: BTreeMap<String, BTreeMap<String, String>>,
    vars_by_id: BTreeMap<String, BTreeMap<String, String>>,
    queries_by_id: BTreeMap<String, String>,
    recipes: BTreeMap<String, Recipe>,
    diagnostics: Vec<RecipeDiagnostic>,
}

impl RecipeRegistry {
    pub fn new(workspace_root: PathBuf, codemod_root: PathBuf) -> Self {
        Self {
            workspace_root,
            codemod_root,
            language_registry: codemod_recipe_engine::LanguageRegistry::new(),
            ast_cache: crate::query::ast_cache::AstParseCache::default(),
            maps_by_id: BTreeMap::new(),
            vars_by_id: BTreeMap::new(),
            queries_by_id: BTreeMap::new(),
            recipes: BTreeMap::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn codemod_root(&self) -> &Path {
        &self.codemod_root
    }

    pub fn maps_by_id(&self) -> &BTreeMap<String, BTreeMap<String, String>> {
        &self.maps_by_id
    }

    pub fn vars_by_id(&self) -> &BTreeMap<String, BTreeMap<String, String>> {
        &self.vars_by_id
    }

    pub fn queries_by_id(&self) -> &BTreeMap<String, String> {
        &self.queries_by_id
    }

    pub fn maps_count(&self) -> usize {
        self.maps_by_id.len()
    }

    pub fn map_ids(&self) -> Vec<String> {
        self.maps_by_id.keys().cloned().collect()
    }

    pub fn var_ids(&self) -> Vec<String> {
        self.vars_by_id.keys().cloned().collect()
    }

    pub fn recipes_map(&self) -> &BTreeMap<String, Recipe> {
        &self.recipes
    }

    pub fn recipes_ast(&self) -> &BTreeMap<String, Recipe> {
        &self.recipes
    }

    pub fn language_registry(&mut self) -> &mut codemod_recipe_engine::LanguageRegistry {
        &mut self.language_registry
    }

    pub fn reload(&mut self) {
        self.maps_by_id.clear();
        self.vars_by_id.clear();
        self.queries_by_id.clear();
        self.recipes.clear();
        self.diagnostics.clear();
        self.ast_cache.clear();

        let files = collect_lua_files(&self.codemod_root);
        for path in files {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let relative = relative_path(&self.workspace_root, &path);
            let name = path.to_string_lossy();
            if path_is_under(&path, &self.codemod_root.join("maps")) {
                match load_map(&text, &name) {
                    Ok(asset) => {
                        self.maps_by_id.insert(asset.id, asset.map);
                    }
                    Err(err) => self.push_load_error(&relative, err),
                }
            } else if path_is_under(&path, &self.codemod_root.join("variables")) {
                match load_variables(&text, &name) {
                    Ok(asset) => {
                        self.vars_by_id.insert(asset.id, asset.values);
                    }
                    Err(err) => self.push_load_error(&relative, err),
                }
            } else if path_is_under(&path, &self.codemod_root.join("queries")) {
                match load_query_lib(&text, &name) {
                    Ok(asset) => {
                        for (k, v) in asset.queries {
                            self.queries_by_id.insert(format!("{}.{}", asset.id, k), v);
                        }
                    }
                    Err(err) => self.push_load_error(&relative, err),
                }
            } else {
                match load_recipe(&text, &name) {
                    Ok(mut recipe) => {
                        recipe.path = Some(path.clone());
                        for (k, v) in recipe.maps.clone() {
                            self.maps_by_id.entry(k).or_insert(v);
                        }
                        if self.recipes.contains_key(&recipe.id) {
                            self.diagnostics.push(RecipeDiagnostic::simple(
                                "error",
                                "E_DUP_ID",
                                format!("Duplicate recipe id {}", recipe.id),
                                vec![source_file_only(&relative)],
                            ));
                        } else {
                            self.recipes.insert(recipe.id.clone(), recipe);
                        }
                    }
                    Err(err) => self.push_load_error(&relative, err),
                }
            }
        }
    }

    fn push_load_error(&mut self, relative: &str, err: LoadError) {
        let (line, column) = match err.location() {
            Some((l, c)) => (Some(l), Some(c)),
            None => (None, None),
        };
        self.diagnostics.push(RecipeDiagnostic {
            severity: "error",
            code: "E_RECIPE_PARSE",
            message: err.to_string(),
            sources: vec![crate::protocol::DiagnosticSource {
                file: relative.to_string(),
                line,
                column,
            }],
            hint: None,
            related_recipe: None,
        });
    }

    pub fn list(&self) -> (Vec<RecipeSchema>, Vec<RecipeDiagnostic>) {
        let recipes = self.recipes.values().map(|r| self.schema_for(r)).collect();
        (recipes, self.diagnostics.clone())
    }

    pub fn get(&self, id: &str) -> Option<RecipeSchema> {
        self.recipes.get(id).map(|r| self.schema_for(r))
    }

    pub fn load_recipe_ast(&self, id: &str) -> Result<(Recipe, PathBuf), String> {
        let recipe = self
            .recipes
            .get(id)
            .cloned()
            .ok_or_else(|| format!("Recipe not found: {id}"))?;
        let path = recipe
            .path
            .clone()
            .unwrap_or_else(|| self.codemod_root.join("recipes"));
        Ok((recipe, path))
    }

    pub fn diagnostics(&self) -> &[RecipeDiagnostic] {
        &self.diagnostics
    }

    fn schema_for(&self, recipe: &Recipe) -> RecipeSchema {
        let relative = recipe
            .path
            .as_ref()
            .map(|p| relative_path(&self.workspace_root, p));
        RecipeSchema {
            id: recipe.id.clone(),
            name: recipe.name.clone().unwrap_or_else(|| recipe.id.clone()),
            description: recipe.description.clone().unwrap_or_default(),
            source_file: relative,
            args: recipe.args.iter().map(arg_schema).collect(),
            explorer_menu: if recipe.explorer_menu.is_empty() {
                None
            } else {
                Some(recipe.explorer_menu.iter().map(explorer_schema).collect())
            },
        }
    }
}

fn arg_schema(arg: &Arg) -> RecipeArg {
    RecipeArg {
        name: arg.name.clone(),
        abbr: arg.abbr.clone(),
        help: arg.help.clone(),
        required: arg.required,
        defaults_to: arg.defaults_to.clone(),
        input_kind: arg.input_kind.clone().unwrap_or_else(|| "text".to_string()),
        options: arg.options.clone(),
        allow_custom_value: arg.allow_custom_value.unwrap_or(false),
        context_key: None,
        from: arg.from.as_ref().map(|f| match f {
            ArgFrom::Builtin(s) => serde_json::Value::String(s.clone()),
            ArgFrom::Spec(spec) => serde_json::json!({
                "template": spec.template,
                "query": spec.query,
                "capture": spec.capture,
            }),
        }),
        from_recipes: vec![],
    }
}

fn explorer_schema(entry: &ExplorerMenuEntry) -> ExplorerMenuEntrySchema {
    ExplorerMenuEntrySchema {
        kind: entry.kind.as_str().to_string(),
        if_expr: entry.if_expr.clone(),
        args: entry.args.clone(),
    }
}

fn collect_lua_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    visit(root, &mut out);
    out.sort();
    out
}

fn visit(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, out);
        } else if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("luau") | Some("lua")
        ) {
            out.push(path);
        }
    }
}

fn path_is_under(path: &Path, dir: &Path) -> bool {
    path.starts_with(dir)
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
