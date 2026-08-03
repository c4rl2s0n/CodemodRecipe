//! Filter recipes for the VS Code Explorer Codemod Recipe QuickPick.

use std::collections::BTreeMap;
use std::sync::Arc;

use codemod_recipe_core::resource_path::resolve_under_root;
use codemod_recipe_yaml::dsl::recipe::arg::field::input_kind::value as input_kind;
use codemod_recipe_yaml::model::Recipe;
use codemod_recipe_yaml::{ExplorerMenuEntry, ExplorerMenuKind};

use crate::path_filters::{path_basename, path_parent, path_stem};
use crate::protocol::{ExplorerRecipeMatch, FilterExplorerRecipesResponse};
use crate::registry::RecipeRegistry;
use crate::step_condition::{condition_expr_passes, eval_string_expr};
use crate::template::PathExistsFn;

/// Build MiniJinja context for explorerMenu `if` / `args` (click path only + derived parts).
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
            // Folder click: treat dirname as the folder itself for parity with extension.
            ctx.insert("fileDirname".to_string(), path);
        }
    }
    ctx
}

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
            matches: None,
        };
    };

    let workspace_root = registry.workspace_root.to_string_lossy().replace('\\', "/");
    let context = build_explorer_context(&workspace_root, path, click_kind);
    let maps = BTreeMap::new();
    let vars = BTreeMap::new();
    let workspace = registry.workspace_root.clone();
    let path_exists: PathExistsFn = Arc::new(move |p: &str| {
        // Absolute paths come from explorer context (`absolutePath`); relative
        // paths must stay under the workspace root.
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

/// First matching entry for `click_kind` → resolved args, or `None` if no match / bind error.
fn resolve_explorer_match(
    recipe: &Recipe,
    click_kind: ExplorerMenuKind,
    path: &str,
    context: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    path_exists: PathExistsFn,
) -> Option<BTreeMap<String, String>> {
    let menu = recipe.explorer_menu.as_ref()?;
    for entry in menu.entries_for_kind(click_kind) {
        let if_expr = entry
            .if_expr
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());
        match condition_expr_passes(if_expr, context, maps, vars, path_exists.clone()) {
            Ok(true) => {
                // Fail closed for whole match on arg resolution errors.
                return resolve_entry_args(recipe, entry, path, context, maps, vars, path_exists)
                    .ok();
            }
            Ok(false) => continue,
            Err(_) => continue,
        }
    }
    None
}

fn resolve_entry_args(
    recipe: &Recipe,
    entry: &ExplorerMenuEntry,
    path: &str,
    context: &BTreeMap<String, String>,
    maps: &BTreeMap<String, BTreeMap<String, String>>,
    vars: &BTreeMap<String, BTreeMap<String, String>>,
    path_exists: PathExistsFn,
) -> Result<BTreeMap<String, String>, String> {
    if entry.args.is_empty() {
        return Ok(input_kind_heuristic(recipe, entry.kind, path));
    }
    let mut out = BTreeMap::new();
    for (name, expr) in &entry.args {
        let value = eval_string_expr(expr, context, maps, vars, path_exists.clone())?;
        out.insert(name.clone(), value);
    }
    Ok(out)
}

fn input_kind_heuristic(
    recipe: &Recipe,
    kind: ExplorerMenuKind,
    path: &str,
) -> BTreeMap<String, String> {
    let want = match kind {
        ExplorerMenuKind::File => input_kind::FILE,
        ExplorerMenuKind::Folder => input_kind::DIRECTORY,
    };
    let mut out = BTreeMap::new();
    if let Some(arg) = recipe
        .args
        .iter()
        .find(|a| a.input_kind.as_deref().map(|k| k == want).unwrap_or(false))
    {
        out.insert(arg.name.clone(), path.to_string());
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use codemod_recipe_yaml::model::Arg;
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

    fn entry(kind: ExplorerMenuKind, if_expr: Option<&str>) -> ExplorerMenuEntry {
        ExplorerMenuEntry {
            kind,
            if_expr: if_expr.map(str::to_string),
            args: BTreeMap::new(),
        }
    }

    #[test]
    fn or_same_kind_matches_once() {
        let recipe = recipe_with_menu(
            "demo",
            ExplorerMenu {
                entries: vec![
                    entry(
                        ExplorerMenuKind::Folder,
                        Some("path is startingwith(\"lib/generated/\")"),
                    ),
                    entry(
                        ExplorerMenuKind::Folder,
                        Some("path is startingwith(\"lib/\")"),
                    ),
                ],
            },
        );
        let mut ctx = BTreeMap::new();
        ctx.insert("path".to_string(), "lib/features/foo".to_string());
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        let exists = Arc::new(|_: &str| false) as PathExistsFn;
        assert!(resolve_explorer_match(
            &recipe,
            ExplorerMenuKind::Folder,
            "lib/features/foo",
            &ctx,
            &maps,
            &vars,
            exists
        )
        .is_some());
    }

    #[test]
    fn missing_if_always_matches_kind() {
        let recipe = recipe_with_menu(
            "demo",
            ExplorerMenu {
                entries: vec![entry(ExplorerMenuKind::File, None)],
            },
        );
        let mut ctx = BTreeMap::new();
        ctx.insert("path".to_string(), "any/where.dart".to_string());
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        let exists = Arc::new(|_: &str| false) as PathExistsFn;
        assert!(resolve_explorer_match(
            &recipe,
            ExplorerMenuKind::File,
            "any/where.dart",
            &ctx,
            &maps,
            &vars,
            exists.clone()
        )
        .is_some());
        assert!(resolve_explorer_match(
            &recipe,
            ExplorerMenuKind::Folder,
            "any/where.dart",
            &ctx,
            &maps,
            &vars,
            exists
        )
        .is_none());
    }

    #[test]
    fn args_expressions_use_path_filters() {
        let mut args = BTreeMap::new();
        args.insert("file".to_string(), "path".to_string());
        args.insert("featureDir".to_string(), "path | parent".to_string());
        args.insert(
            "folderName".to_string(),
            "path | parent | basename".to_string(),
        );
        // Also bind recipe arg named `path` from path expression — LHS/RHS scoping.
        args.insert("path".to_string(), "path | parent".to_string());

        let recipe = recipe_with_menu(
            "demo",
            ExplorerMenu {
                entries: vec![ExplorerMenuEntry {
                    kind: ExplorerMenuKind::File,
                    if_expr: None,
                    args,
                }],
            },
        );
        let click = "lib/features/foo.dart";
        let mut ctx = BTreeMap::new();
        ctx.insert("path".to_string(), click.to_string());
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        let exists = Arc::new(|_: &str| false) as PathExistsFn;
        let resolved = resolve_explorer_match(
            &recipe,
            ExplorerMenuKind::File,
            click,
            &ctx,
            &maps,
            &vars,
            exists,
        )
        .expect("match");
        assert_eq!(resolved.get("file").map(String::as_str), Some(click));
        assert_eq!(
            resolved.get("featureDir").map(String::as_str),
            Some("lib/features")
        );
        assert_eq!(
            resolved.get("folderName").map(String::as_str),
            Some("features")
        );
        assert_eq!(
            resolved.get("path").map(String::as_str),
            Some("lib/features")
        );
    }

    #[test]
    fn first_matching_entry_args_win() {
        let mut first_args = BTreeMap::new();
        first_args.insert("tag".to_string(), "\"first\"".to_string());
        let mut second_args = BTreeMap::new();
        second_args.insert("tag".to_string(), "\"second\"".to_string());

        let recipe = recipe_with_menu(
            "demo",
            ExplorerMenu {
                entries: vec![
                    ExplorerMenuEntry {
                        kind: ExplorerMenuKind::File,
                        if_expr: Some("path is startingwith(\"lib/\")".into()),
                        args: first_args,
                    },
                    ExplorerMenuEntry {
                        kind: ExplorerMenuKind::File,
                        if_expr: None,
                        args: second_args,
                    },
                ],
            },
        );
        let click = "lib/a.dart";
        let mut ctx = BTreeMap::new();
        ctx.insert("path".to_string(), click.to_string());
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        let exists = Arc::new(|_: &str| false) as PathExistsFn;
        let resolved = resolve_explorer_match(
            &recipe,
            ExplorerMenuKind::File,
            click,
            &ctx,
            &maps,
            &vars,
            exists,
        )
        .expect("match");
        assert_eq!(resolved.get("tag").map(String::as_str), Some("first"));
    }

    #[test]
    fn input_kind_heuristic_when_args_absent() {
        let recipe = Recipe {
            id: "demo".to_string(),
            name: None,
            description: None,
            args: vec![Arg {
                name: "directory".to_string(),
                required: true,
                input_kind: Some(input_kind::DIRECTORY.to_string()),
                abbr: None,
                help: None,
                defaults_to: None,
                options: vec![],
                allow_custom_value: None,
                context_key: None,
                from: None,
            }],
            maps: BTreeMap::new(),
            queries: BTreeMap::new(),
            steps: vec![],
            post_execution: vec![],
            explorer_menu: Some(ExplorerMenu {
                entries: vec![entry(ExplorerMenuKind::Folder, None)],
            }),
        };
        let click = "lib/features";
        let mut ctx = BTreeMap::new();
        ctx.insert("path".to_string(), click.to_string());
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        let exists = Arc::new(|_: &str| false) as PathExistsFn;
        let resolved = resolve_explorer_match(
            &recipe,
            ExplorerMenuKind::Folder,
            click,
            &ctx,
            &maps,
            &vars,
            exists,
        )
        .expect("match");
        assert_eq!(
            resolved.get("directory").map(String::as_str),
            Some("lib/features")
        );
    }

    #[test]
    fn bind_error_omits_recipe() {
        let mut args = BTreeMap::new();
        args.insert("x".to_string(), "undefined_filter | nope".to_string());
        let recipe = recipe_with_menu(
            "demo",
            ExplorerMenu {
                entries: vec![ExplorerMenuEntry {
                    kind: ExplorerMenuKind::File,
                    if_expr: None,
                    args,
                }],
            },
        );
        let mut ctx = BTreeMap::new();
        ctx.insert("path".to_string(), "lib/a.dart".to_string());
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        let exists = Arc::new(|_: &str| false) as PathExistsFn;
        assert!(resolve_explorer_match(
            &recipe,
            ExplorerMenuKind::File,
            "lib/a.dart",
            &ctx,
            &maps,
            &vars,
            exists
        )
        .is_none());
    }

    #[test]
    fn context_exposes_path_parts_and_absolute() {
        let ctx = build_explorer_context("/ws", "lib/features/foo.dart", ExplorerMenuKind::File);
        assert_eq!(
            ctx.get("path").map(String::as_str),
            Some("lib/features/foo.dart")
        );
        assert_eq!(
            ctx.get("absolutePath").map(String::as_str),
            Some("/ws/lib/features/foo.dart")
        );
        assert_eq!(ctx.get("workspaceRoot").map(String::as_str), Some("/ws"));
        assert_eq!(
            ctx.get("fileBasename").map(String::as_str),
            Some("foo.dart")
        );
        assert_eq!(ctx.get("fileStem").map(String::as_str), Some("foo"));
        assert_eq!(ctx.get("fileExt").map(String::as_str), Some("dart"));
        assert_eq!(
            ctx.get("fileDirname").map(String::as_str),
            Some("lib/features")
        );
        assert_eq!(
            ctx.get("file").map(String::as_str),
            Some("lib/features/foo.dart")
        );
        assert_eq!(
            ctx.get("directory").map(String::as_str),
            Some("lib/features")
        );
    }

    #[test]
    fn args_use_named_builtins() {
        let mut args = BTreeMap::new();
        args.insert("file".to_string(), "path".to_string());
        args.insert("folderName".to_string(), "fileBasename".to_string());
        args.insert("featureDir".to_string(), "fileDirname".to_string());
        args.insert("absFile".to_string(), "absolutePath".to_string());

        let recipe = recipe_with_menu(
            "demo",
            ExplorerMenu {
                entries: vec![ExplorerMenuEntry {
                    kind: ExplorerMenuKind::File,
                    if_expr: Some("fileExt == \"dart\"".into()),
                    args,
                }],
            },
        );
        let click = "lib/features/foo.dart";
        let ctx = build_explorer_context("/ws", click, ExplorerMenuKind::File);
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        let exists = Arc::new(|_: &str| false) as PathExistsFn;
        let resolved = resolve_explorer_match(
            &recipe,
            ExplorerMenuKind::File,
            click,
            &ctx,
            &maps,
            &vars,
            exists,
        )
        .expect("match");
        assert_eq!(resolved.get("file").map(String::as_str), Some(click));
        assert_eq!(
            resolved.get("folderName").map(String::as_str),
            Some("foo.dart")
        );
        assert_eq!(
            resolved.get("featureDir").map(String::as_str),
            Some("lib/features")
        );
        assert_eq!(
            resolved.get("absFile").map(String::as_str),
            Some("/ws/lib/features/foo.dart")
        );
    }

    #[test]
    fn if_file_ext_filters_non_dart() {
        let recipe = recipe_with_menu(
            "demo",
            ExplorerMenu {
                entries: vec![ExplorerMenuEntry {
                    kind: ExplorerMenuKind::File,
                    if_expr: Some("fileExt == \"dart\"".into()),
                    args: BTreeMap::new(),
                }],
            },
        );
        let maps = BTreeMap::new();
        let vars = BTreeMap::new();
        let exists = Arc::new(|_: &str| false) as PathExistsFn;
        let dart_ctx = build_explorer_context("/ws", "lib/a.dart", ExplorerMenuKind::File);
        assert!(resolve_explorer_match(
            &recipe,
            ExplorerMenuKind::File,
            "lib/a.dart",
            &dart_ctx,
            &maps,
            &vars,
            exists.clone()
        )
        .is_some());
        let md_ctx = build_explorer_context("/ws", "README.md", ExplorerMenuKind::File);
        assert!(resolve_explorer_match(
            &recipe,
            ExplorerMenuKind::File,
            "README.md",
            &md_ctx,
            &maps,
            &vars,
            exists
        )
        .is_none());
    }
}
