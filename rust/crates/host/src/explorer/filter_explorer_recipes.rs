//! Filter recipes for the VS Code Explorer Codemod Recipe QuickPick.

use std::collections::BTreeMap;
use std::sync::Arc;

use codemod_recipe_core::resource_path::resolve_under_root;
use codemod_recipe_lua::model::{ExplorerMenuKind, Recipe};
use codemod_recipe_lua::InputKind;

use crate::engine::step_condition::{condition_expr_passes, eval_string_expr};
use crate::protocol::{ExplorerRecipeMatch, FilterExplorerRecipesResponse};
use crate::recipe::registry::RecipeRegistry;
use crate::template::path_filters::{path_basename, path_parent, path_stem};
use crate::template::PathExistsFn;

pub fn build_explorer_context(
    workspace_root: &str,
    relative_path: &str,
    kind: ExplorerMenuKind,
) -> BTreeMap<String, String> {
    let path = relative_path.replace('\\', "/");
    let path = path.trim_end_matches('/').to_string();
    let root = workspace_root.replace('\\', "/");
    let root = root.trim_end_matches('/').to_string();

    let file_basename = path_basename(&path);
    let file_stem = path_stem(&path);
    let file_dirname = path_parent(&path);
    let file_ext = {
        let base = &file_basename;
        match base.rfind('.') {
            Some(i) if i > 0 => base[i + 1..].to_string(),
            _ => String::new(),
        }
    };

    let absolute_path = if std::path::Path::new(&path).is_absolute() {
        path.clone()
    } else if path.is_empty() {
        root.clone()
    } else {
        format!("{root}/{path}")
    };

    let mut ctx = BTreeMap::new();
    ctx.insert("path".to_string(), path.clone());
    ctx.insert("absolutePath".to_string(), absolute_path);
    ctx.insert("workspaceRoot".to_string(), root);
    ctx.insert("fileBasename".to_string(), file_basename);
    ctx.insert("fileStem".to_string(), file_stem);
    ctx.insert("fileExt".to_string(), file_ext);
    ctx.insert("fileDirname".to_string(), file_dirname.clone());

    match kind {
        ExplorerMenuKind::File => {
            ctx.insert("file".to_string(), path);
            ctx.insert("directory".to_string(), file_dirname);
        }
        ExplorerMenuKind::Folder => {
            ctx.insert("file".to_string(), String::new());
            ctx.insert("directory".to_string(), path.clone());
            ctx.insert("fileDirname".to_string(), path);
        }
    }
    ctx
}

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
            matches: None,
        };
    };

    let workspace_root = registry.workspace_root.to_string_lossy().replace('\\', "/");
    let context = build_explorer_context(&workspace_root, path, click_kind);
    let maps = BTreeMap::new();
    let vars = BTreeMap::new();
    let workspace = registry.workspace_root.clone();
    let path_exists: PathExistsFn = Arc::new(move |p: &str| {
        if std::path::Path::new(p).is_absolute() {
            return std::path::Path::new(p).exists();
        }
        match resolve_under_root(&workspace, p) {
            Ok(absolute) => absolute.exists(),
            Err(_) => false,
        }
    });

    let mut matches = Vec::new();
    for (id, recipe) in registry.recipes_ast() {
        if let Some(args) = resolve_explorer_match(
            recipe,
            click_kind,
            path,
            &context,
            &maps,
            &vars,
            path_exists.clone(),
        ) {
            matches.push(ExplorerRecipeMatch {
                recipe_id: id.clone(),
                args,
            });
        }
    }
    matches.sort_by(|a, b| a.recipe_id.cmp(&b.recipe_id));

    FilterExplorerRecipesResponse {
        ok: true,
        error: None,
        matches: Some(matches),
    }
}

fn resolve_explorer_match(
    recipe: &Recipe,
    click_kind: ExplorerMenuKind,
    path: &str,
    context: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    path_exists: PathExistsFn,
) -> Option<BTreeMap<String, String>> {
    for entry in recipe.explorer_menu.iter().filter(|e| e.kind == click_kind) {
        let if_expr = entry
            .if_expr
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        match condition_expr_passes(
            if_expr,
            &crate::recipe::args::strings_to_arg_map(context),
            maps,
            vars,
            path_exists.clone(),
        ) {
            Ok(true) => {
                if entry.args.is_empty() {
                    return Some(input_kind_heuristic(recipe, entry.kind, path));
                }
                return resolve_entry_args(&entry.args, context, maps, vars, path_exists).ok();
            }
            Ok(false) => continue,
            Err(_) => continue,
        }
    }
    None
}

fn resolve_entry_args(
    entry_args: &BTreeMap<String, String>,
    context: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    path_exists: PathExistsFn,
) -> Result<BTreeMap<String, String>, String> {
    let mut out = BTreeMap::new();
    for (name, expr) in entry_args {
        let value = eval_string_expr(
            expr,
            &crate::recipe::args::strings_to_arg_map(context),
            maps,
            vars,
            path_exists.clone(),
        )?;
        out.insert(name.clone(), value);
    }
    Ok(out)
}

fn input_kind_heuristic(
    recipe: &Recipe,
    kind: ExplorerMenuKind,
    path: &str,
) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    if let Some(arg) = recipe.args.iter().find(|a| {
        matches!(
            (kind, &a.input_kind),
            (ExplorerMenuKind::File, InputKind::File)
                | (ExplorerMenuKind::Folder, InputKind::Directory)
        )
    }) {
        out.insert(arg.name.clone(), path.to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use codemod_recipe_lua::model::{Arg, ExplorerMenuEntry, Recipe};
    use codemod_recipe_lua::InputKind;

    fn recipe_with_menu(id: &str, kind: ExplorerMenuKind) -> Recipe {
        Recipe {
            id: id.to_string(),
            name: None,
            description: None,
            args: vec![Arg {
                name: "file".to_string(),
                required: true,
                input_kind: InputKind::File,
                abbr: None,
                help: None,
                defaults_to: None,
                options: vec![],
                options_from_map: None,
                allow_custom_value: None,
                from: None,
            }],
            maps: BTreeMap::new(),
            types: BTreeMap::new(),
            queries: BTreeMap::new(),
            post_execution: vec![],
            explorer_menu: vec![ExplorerMenuEntry {
                kind,
                if_expr: None,
                args: BTreeMap::new(),
            }],
            source: String::new(),
            path: None,
        }
    }

    #[test]
    fn file_click_binds_file_arg() {
        let recipe = recipe_with_menu("demo", ExplorerMenuKind::File);
        let args = resolve_explorer_match(
            &recipe,
            ExplorerMenuKind::File,
            "lib/a.dart",
            &BTreeMap::new(),
            &BTreeMap::new(),
            &BTreeMap::new(),
            Arc::new(|_| true),
        )
        .unwrap();
        assert_eq!(args.get("file").map(String::as_str), Some("lib/a.dart"));
    }
}
