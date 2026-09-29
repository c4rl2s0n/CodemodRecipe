use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use codemod_recipe_lua::load::{load_asset_with_root, Asset, LoadError, LoadResult};
use codemod_recipe_lua::model::{
    Arg, ArgFrom, ComponentDef, ExplorerMenuEntry, Recipe, StackDef, TestSuiteDef, TypeDef,
};
use codemod_recipe_lua::require_chunk_name;
use codemod_recipe_lua::InputKind;

use crate::diagnostics::diag_source::source_file_only;
use crate::protocol::{
    ComponentDetailSchema, ComponentOperationSchema, ComponentSchema, ExplorerMenuEntrySchema,
    RecipeArg, RecipeDiagnostic, RecipeSchema, StackSchema, TypeSchema,
};

pub struct RecipeRegistry {
    pub workspace_root: PathBuf,
    codemod_root: PathBuf,
    pub language_registry: codemod_recipe_engine::LanguageRegistry,
    pub ast_cache: crate::query::ast_cache::AstParseCache,
    maps_by_id: BTreeMap<String, BTreeMap<String, String>>,
    vars_by_id: BTreeMap<String, BTreeMap<String, String>>,
    queries_by_id: BTreeMap<String, String>,
    types_by_id: BTreeMap<String, TypeDef>,
    recipes: BTreeMap<String, Recipe>,
    stack: Option<StackDef>,
    components: BTreeMap<String, ComponentDef>,
    test_suites: BTreeMap<String, TestSuiteDef>,
    component_scan_cache: Option<crate::component::cache::ComponentScanCache>,
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
            types_by_id: BTreeMap::new(),
            recipes: BTreeMap::new(),
            stack: None,
            components: BTreeMap::new(),
            test_suites: BTreeMap::new(),
            component_scan_cache: None,
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

    pub fn types_by_id(&self) -> &BTreeMap<String, TypeDef> {
        &self.types_by_id
    }

    pub fn type_ids(&self) -> Vec<String> {
        self.types_by_id.keys().cloned().collect()
    }

    pub fn type_schemas(&self) -> Vec<TypeSchema> {
        self.types_by_id
            .values()
            .map(|t| type_schema(t, &self.maps_by_id, &self.types_by_id))
            .collect()
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

    pub fn stack(&self) -> Option<&StackDef> {
        self.stack.as_ref()
    }

    pub fn components(&self) -> &BTreeMap<String, ComponentDef> {
        &self.components
    }

    pub fn get_component(&self, id: &str) -> Option<&ComponentDef> {
        self.components.get(id)
    }

    pub fn test_suites(&self) -> &BTreeMap<String, TestSuiteDef> {
        &self.test_suites
    }

    pub fn get_test_suite(&self, id: &str) -> Option<&TestSuiteDef> {
        self.test_suites.get(id)
    }

    pub fn component_schemas(&self) -> Vec<ComponentSchema> {
        let root_ids: std::collections::BTreeSet<&str> = self
            .stack
            .as_ref()
            .map(|s| s.components.iter().map(String::as_str).collect())
            .unwrap_or_default();
        self.components
            .values()
            .map(|c| self.component_schema(c, root_ids.contains(c.id.as_str())))
            .collect()
    }

    pub fn stack_schema(&self) -> Option<StackSchema> {
        self.stack.as_ref().map(|s| StackSchema {
            id: s.id.clone(),
            description: s.description.clone(),
            components: s.components.clone(),
            codebase: s.codebase.clone(),
        })
    }

    pub fn describe_component(&self, id: &str) -> Option<ComponentDetailSchema> {
        let c = self.components.get(id)?;
        let is_root = self
            .stack
            .as_ref()
            .map(|s| s.components.iter().any(|cid| cid == id))
            .unwrap_or(false);
        Some(self.component_detail(c, is_root))
    }

    pub fn workspace_root(&self) -> &Path {
        &self.workspace_root
    }

    pub fn component_scan_cache(&self) -> Option<&crate::component::cache::ComponentScanCache> {
        self.component_scan_cache.as_ref()
    }

    pub fn component_scan_cache_mut(
        &mut self,
    ) -> &mut Option<crate::component::cache::ComponentScanCache> {
        &mut self.component_scan_cache
    }

    pub fn language_registry(&mut self) -> &mut codemod_recipe_engine::LanguageRegistry {
        &mut self.language_registry
    }

    pub fn reload(&mut self) {
        self.maps_by_id.clear();
        self.vars_by_id.clear();
        self.queries_by_id.clear();
        self.types_by_id.clear();
        self.recipes.clear();
        self.stack = None;
        self.components.clear();
        self.test_suites.clear();
        self.component_scan_cache = None;
        self.diagnostics.clear();
        self.ast_cache.clear();

        let files = collect_lua_files(&self.codemod_root);
        for path in files {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let relative = relative_path(&self.workspace_root, &path);
            let name = require_chunk_name(&path);
            match load_asset_with_root(&text, &name, Some(&self.codemod_root)) {
                Ok(LoadResult::Asset(asset)) => self.register_asset(*asset, &path, &relative),
                Ok(LoadResult::Library) => {}
                Err(err) => self.push_load_error(&relative, err),
            }
        }
        self.validate_stack();
    }

    fn validate_stack(&mut self) {
        if self.components.is_empty() {
            return;
        }
        let Some(stack) = self.stack.as_ref() else {
            self.diagnostics.push(RecipeDiagnostic::simple(
                "error",
                "E_STACK_MISSING",
                "Missing stack manifest at .codemod/stack.luau".to_string(),
                vec![source_file_only(".codemod/stack.luau")],
            ));
            return;
        };
        for component_id in &stack.components {
            if !self.components.contains_key(component_id) {
                let stack_file = stack
                    .path
                    .as_ref()
                    .map(|p| relative_path(&self.workspace_root, p))
                    .unwrap_or_else(|| ".codemod/stack.luau".to_string());
                self.diagnostics.push(RecipeDiagnostic::simple(
                    "error",
                    "E_STACK_COMPONENT",
                    format!(
                        "stack {} references unknown component {}",
                        stack.id, component_id
                    ),
                    vec![source_file_only(&stack_file)],
                ));
            }
        }
    }

    fn register_asset(&mut self, asset: Asset, path: &Path, relative: &str) {
        match asset {
            Asset::Recipe(mut recipe) => {
                recipe.path = Some(path.to_path_buf());
                for (k, v) in recipe.maps.clone() {
                    self.maps_by_id.entry(k).or_insert(v);
                }
                for (k, v) in recipe.types.clone() {
                    self.types_by_id.entry(k).or_insert(v);
                }
                if self.recipes.contains_key(&recipe.id) {
                    self.diagnostics.push(RecipeDiagnostic::simple(
                        "error",
                        "E_DUP_ID",
                        format!("Duplicate recipe id {}", recipe.id),
                        vec![source_file_only(relative)],
                    ));
                } else {
                    self.recipes.insert(recipe.id.clone(), recipe);
                }
            }
            Asset::Map(asset) => {
                self.maps_by_id.insert(asset.id, asset.map);
            }
            Asset::Variables(asset) => {
                self.vars_by_id.insert(asset.id, asset.values);
            }
            Asset::QueryLib(asset) => {
                for (k, v) in asset.queries {
                    self.queries_by_id.insert(format!("{}.{}", asset.id, k), v);
                }
            }
            Asset::Type(asset) => {
                if self.types_by_id.contains_key(&asset.id) {
                    self.diagnostics.push(RecipeDiagnostic::simple(
                        "error",
                        "E_DUP_ID",
                        format!("Duplicate type id {}", asset.id),
                        vec![source_file_only(relative)],
                    ));
                } else {
                    let id = asset.id.clone();
                    self.types_by_id.insert(id, asset);
                }
            }
            Asset::Stack(mut stack) => {
                if self.stack.is_some() {
                    self.diagnostics.push(RecipeDiagnostic::simple(
                        "error",
                        "E_DUP_STACK",
                        "Only one stack manifest is allowed (.codemod/stack.luau)".to_string(),
                        vec![source_file_only(relative)],
                    ));
                } else {
                    stack.path = Some(path.to_path_buf());
                    self.stack = Some(stack);
                }
            }
            Asset::Component(mut component) => {
                if self.components.contains_key(&component.id) {
                    self.diagnostics.push(RecipeDiagnostic::simple(
                        "error",
                        "E_DUP_ID",
                        format!("Duplicate component id {}", component.id),
                        vec![source_file_only(relative)],
                    ));
                } else {
                    component.path = Some(path.to_path_buf());
                    self.components.insert(component.id.clone(), component);
                }
            }
            Asset::TestSuite(mut suite) => {
                if self.test_suites.contains_key(&suite.id) {
                    self.diagnostics.push(RecipeDiagnostic::simple(
                        "error",
                        "E_DUP_ID",
                        format!("Duplicate test suite id {}", suite.id),
                        vec![source_file_only(relative)],
                    ));
                } else {
                    suite.path = Some(path.to_path_buf());
                    self.test_suites.insert(suite.id.clone(), suite);
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
            args: recipe
                .args
                .iter()
                .map(|a| arg_schema(a, &self.maps_by_id, &self.types_by_id))
                .collect(),
            explorer_menu: if recipe.explorer_menu.is_empty() {
                None
            } else {
                Some(recipe.explorer_menu.iter().map(explorer_schema).collect())
            },
            post_execution: if recipe.post_execution.is_empty() {
                None
            } else {
                Some(recipe.post_execution.clone())
            },
        }
    }

    fn component_schema(&self, component: &ComponentDef, is_root: bool) -> ComponentSchema {
        let relative = component
            .path
            .as_ref()
            .map(|p| relative_path(&self.workspace_root, p));
        ComponentSchema {
            id: component.id.clone(),
            description: component.description.clone(),
            operations: component
                .operations
                .iter()
                .map(|o| o.name.clone())
                .collect(),
            has_scaffold: component.has_scaffold,
            is_root,
            source_file: relative,
        }
    }

    fn component_detail(&self, component: &ComponentDef, is_root: bool) -> ComponentDetailSchema {
        let relative = component
            .path
            .as_ref()
            .map(|p| relative_path(&self.workspace_root, p));
        ComponentDetailSchema {
            id: component.id.clone(),
            description: component.description.clone(),
            has_identify: component.has_identify,
            has_verify: component.has_verify,
            has_relate: component.has_relate,
            has_to_summary: component.has_to_summary,
            has_scaffold: component.has_scaffold,
            is_root,
            source_file: relative,
            scaffold_args: component
                .scaffold_args
                .iter()
                .map(|a| arg_schema(a, &self.maps_by_id, &self.types_by_id))
                .collect(),
            operations: component
                .operations
                .iter()
                .map(|op| ComponentOperationSchema {
                    name: op.name.clone(),
                    description: op.description.clone(),
                    args: op
                        .args
                        .iter()
                        .map(|a| arg_schema(a, &self.maps_by_id, &self.types_by_id))
                        .collect(),
                })
                .collect(),
        }
    }
}

fn arg_schema(
    arg: &Arg,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    types: &BTreeMap<String, TypeDef>,
) -> RecipeArg {
    let options = crate::recipe::args::choice_options(arg, maps);
    let allow_custom = !matches!(arg.input_kind, InputKind::Choice);
    let fields = arg.input_kind.type_id().and_then(|id| {
        types.get(id).map(|def| {
            def.fields
                .iter()
                .map(|f| arg_schema(f, maps, types))
                .collect()
        })
    });
    RecipeArg {
        name: arg.name.clone(),
        abbr: arg.abbr.clone(),
        help: arg.help.clone(),
        required: arg.required,
        defaults_to: arg.defaults_to.clone(),
        input_kind: arg.input_kind.to_json(),
        options,
        options_from_map: arg.options_from_map.clone(),
        allow_custom_value: arg.allow_custom_value.unwrap_or(allow_custom),
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
        fields,
    }
}

fn type_schema(
    def: &TypeDef,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    types: &BTreeMap<String, TypeDef>,
) -> TypeSchema {
    TypeSchema {
        id: def.id.clone(),
        description: def.description.clone(),
        fields: def
            .fields
            .iter()
            .map(|f| arg_schema(f, maps, types))
            .collect(),
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
    // `.codemod/tests/fixtures/` holds workspace fixtures for test suites —
    // never load them as assets (a fixture may itself contain a .codemod tree).
    let fixtures_root = root.join("tests").join("fixtures");
    out.retain(|p| !p.starts_with(&fixtures_root));
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
        ) && !is_declaration_file(&path)
        {
            out.push(path);
        }
    }
}

fn is_declaration_file(path: &Path) -> bool {
    codemod_recipe_lua::is_declaration_file_name(path)
}

fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
