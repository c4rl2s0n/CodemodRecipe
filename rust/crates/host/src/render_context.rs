//! Bundled recipe/template inputs for host rendering (clippy-friendly).

use std::collections::BTreeMap;
use std::path::Path;

use codemod_recipe_yaml::model::Recipe;

use crate::registry::RecipeRegistry;

/// Template + query resolution context for a recipe invocation.
pub struct RecipeRenderContext<'a> {
    pub recipe: &'a Recipe,
    pub registry: Option<&'a RecipeRegistry>,
    pub recipe_file: Option<&'a Path>,
    pub codemod_root: Option<&'a Path>,
    pub args: &'a BTreeMap<String, String>,
    pub maps: &'a BTreeMap<String, BTreeMap<String, String>>,
    pub vars: &'a BTreeMap<String, BTreeMap<String, String>>,
}

impl<'a> RecipeRenderContext<'a> {
    pub fn with_registry(
        recipe: &'a Recipe,
        registry: &'a RecipeRegistry,
        recipe_file: Option<&'a Path>,
        args: &'a BTreeMap<String, String>,
        maps: &'a BTreeMap<String, BTreeMap<String, String>>,
        vars: &'a BTreeMap<String, BTreeMap<String, String>>,
    ) -> Self {
        Self {
            recipe,
            registry: Some(registry),
            recipe_file,
            codemod_root: Some(registry.codemod_root()),
            args,
            maps,
            vars,
        }
    }
}
