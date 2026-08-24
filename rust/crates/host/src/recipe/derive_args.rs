use crate::protocol::DeriveArgsResponse;
use crate::recipe::registry::RecipeRegistry;
use std::collections::BTreeMap;

pub struct DeriveArgsRequest<'a> {
    pub recipe_id: &'a str,
    pub source: &'a str,
    pub language: Option<&'a str>,
    pub path: Option<&'a str>,
    pub cursor_offset: usize,
    pub selection_start: usize,
    pub selection_end: usize,
    pub context: BTreeMap<String, String>,
}

pub fn derive_args(registry: &RecipeRegistry, req: DeriveArgsRequest<'_>) -> DeriveArgsResponse {
    let Ok((recipe, _)) = registry.load_recipe_ast(req.recipe_id) else {
        return DeriveArgsResponse {
            ok: false,
            error: Some(format!("Recipe not found: {}", req.recipe_id)),
            args: None,
        };
    };
    let mut args = BTreeMap::new();
    for arg in &recipe.args {
        if let Some(codemod_recipe_lua::ArgFrom::Builtin(key)) = &arg.from {
            if let Some(v) = req.context.get(key) {
                args.insert(arg.name.clone(), v.clone());
            }
        }
    }
    DeriveArgsResponse {
        ok: true,
        error: None,
        args: Some(args),
    }
}
