//! Filter recipes for the VS Code Explorer Codemod Recipe QuickPick.

use std::collections::BTreeMap;
use std::sync::Arc;

use codemod_recipe_yaml::model::Recipe;
use codemod_recipe_yaml::ExplorerMenuKind;

use crate::protocol::FilterExplorerRecipesResponse;
use crate::registry::RecipeRegistry;
use crate::step_condition::condition_expr_passes;

/// Recipes whose `explorerMenu` matches Explorer click `kind` + `path`.
pub fn filter_explorer_recipes(
    registry: &RecipeRegistry,
    path: &str,
    kind: &str,
) -> FilterExplorerRecipesResponse {
    let Some(click_kind) = ExplorerMenuKind::parse(kind) else {
        return FilterExplorerRecipesResponse {
            ok: false,
            error: Some(format!(
                "invalid explorer kind '{kind}' (expected file or folder)"
            )),
            recipe_ids: None,
        };
    };

    let mut context = BTreeMap::new();
    context.insert("path".to_string(), path.to_string());
    let maps = BTreeMap::new();
    let vars = BTreeMap::new();
    let workspace = registry.workspace_root.clone();
    let path_exists: Arc<dyn Fn(&str) -> bool + Send + Sync> = Arc::new(move |p: &str| {
        let candidate = if std::path::Path::new(p).is_absolute() {
            std::path::PathBuf::from(p)
        } else {
            workspace.join(p)
        };
        candidate.exists()
    });

    let mut recipe_ids = Vec::new();
    for (id, recipe) in registry.recipes_ast() {
        if recipe_matches_explorer(recipe, click_kind, &context, &maps, &vars, path_exists.clone())
        {
            recipe_ids.push(id.clone());
        }
    }
    recipe_ids.sort();

    FilterExplorerRecipesResponse {
        ok: true,
        error: None,
        recipe_ids: Some(recipe_ids),
    }
}

fn recipe_matches_explorer(
    recipe: &Recipe,
    click_kind: ExplorerMenuKind,
    context: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    path_exists: Arc<dyn Fn(&str) -> bool + Send + Sync>,
) -> bool {
    let Some(menu) = recipe.explorer_menu.as_ref() else {
        return false;
    };
    for entry in menu.entries_for_kind(click_kind) {
        let if_expr = entry
            .if_expr
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        match condition_expr_passes(if_expr, context, maps, vars, path_exists.clone()) {
            Ok(true) => return true,
            Ok(false) => continue,
            Err(_) => continue, // fail closed for this entry; try others
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use codemod_recipe_yaml::{ExplorerMenu, ExplorerMenuEntry};

    fn recipe_with_menu(id: &str, menu: ExplorerMenu) -> Recipe {
        Recipe {
            id: id.to_string(),
            name: None,
            description: None,
            args: vec![],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![],
            post_execution: vec![],
            explorer_menu: Some(menu),
        }
    }

    #[test]
    fn or_same_kind_matches_once() {
        let recipe = recipe_with_menu(
            "demo",
            ExplorerMenu {
                entries: vec![
                    ExplorerMenuEntry {
                        kind: ExplorerMenuKind::Folder,
                        if_expr: Some("path is startingwith(\"lib/generated/\")".into()),
                    },
                    ExplorerMenuEntry {
                        kind: ExplorerMenuKind::Folder,
                        if_expr: Some("path is startingwith(\"lib/\")".into()),
                    },
                ],
            },
        );
        let mut ctx = BTreeMap::new();
        ctx.insert("path".to_string(), "lib/features/foo".to_string());
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        let exists = Arc::new(|_: &str| false) as Arc<dyn Fn(&str) -> bool + Send + Sync>;
        assert!(recipe_matches_explorer(
            &recipe,
            ExplorerMenuKind::Folder,
            &ctx,
            &maps,
            &vars,
            exists
        ));
    }

    #[test]
    fn missing_if_always_matches_kind() {
        let recipe = recipe_with_menu(
            "demo",
            ExplorerMenu {
                entries: vec![ExplorerMenuEntry {
                    kind: ExplorerMenuKind::File,
                    if_expr: None,
                }],
            },
        );
        let mut ctx = BTreeMap::new();
        ctx.insert("path".to_string(), "any/where.dart".to_string());
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        let exists = Arc::new(|_: &str| false) as Arc<dyn Fn(&str) -> bool + Send + Sync>;
        assert!(recipe_matches_explorer(
            &recipe,
            ExplorerMenuKind::File,
            &ctx,
            &maps,
            &vars,
            exists.clone()
        ));
        assert!(!recipe_matches_explorer(
            &recipe,
            ExplorerMenuKind::Folder,
            &ctx,
            &maps,
            &vars,
            exists
        ));
    }
}
