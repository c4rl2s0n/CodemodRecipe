//! HostCommand dispatch: thin match arms delegate to per-domain handlers.

mod catalog;
mod common;
mod components;
mod query;
mod recipe;

use crate::protocol::dispatch::common::RecipeRequest;
use crate::protocol::HostCommand;
use crate::recipe::registry::RecipeRegistry;

/// Route a HostCommand to its domain handler.
pub fn handle_command(registry: &mut RecipeRegistry, cmd: HostCommand) -> serde_json::Value {
    match cmd {
        HostCommand::Reload => catalog::reload(registry),
        HostCommand::List => catalog::list(registry),
        HostCommand::Validate { recipe } => catalog::validate(registry, recipe),
        HostCommand::Describe { recipe } => catalog::describe(registry, recipe),
        HostCommand::GenerateAstPath { path, offset } => catalog::generate_ast_path(path, offset),
        HostCommand::Bootstrap {
            force,
            edit_policy,
            companions,
        } => catalog::bootstrap(registry, force, edit_policy, companions),
        HostCommand::DeriveArgs {
            recipe,
            source,
            language,
            path,
            cursor_offset,
            selection_start,
            selection_end,
            context,
        } => query::derive_args(
            registry,
            recipe,
            source,
            language,
            path,
            cursor_offset,
            selection_start,
            selection_end,
            context,
        ),
        HostCommand::FilterExplorerRecipes { path, kind } => {
            query::filter_explorer_recipes(registry, path, kind)
        }
        HostCommand::DumpAst {
            path,
            source,
            language,
            named_only,
        } => query::dump_ast(registry, path, source, language, named_only),
        HostCommand::DebugQuery {
            path,
            source,
            language,
            query,
            instrument,
            include_sexp,
        } => query::debug_query(
            registry,
            path,
            source,
            language,
            query,
            instrument,
            include_sexp,
        ),
        HostCommand::GenerateQuery {
            path,
            source,
            language,
            start,
            end,
            include_text_predicates,
            capture_leaf,
            max_depth,
        } => query::generate_query(
            registry,
            path,
            source,
            language,
            start,
            end,
            include_text_predicates,
            capture_leaf,
            max_depth,
        ),
        HostCommand::ResolveStaticPath { template } => query::resolve_static_path(template),
        HostCommand::QueryCaptures { query, query_id } => {
            query::query_captures(registry, query, query_id)
        }
        HostCommand::Preview {
            recipe,
            inline_recipe,
            args,
            snippet_lines,
        } => recipe::preview(
            registry,
            RecipeRequest {
                recipe_id: recipe.as_deref(),
                inline_recipe: inline_recipe.as_ref(),
                args: &args,
            },
            true,
            snippet_lines,
        ),
        HostCommand::Apply {
            recipe,
            inline_recipe,
            args,
            preview_token,
            selection,
        } => recipe::apply(
            registry,
            RecipeRequest {
                recipe_id: recipe.as_deref(),
                inline_recipe: inline_recipe.as_ref(),
                args: &args,
            },
            &preview_token,
            &selection,
        ),
        HostCommand::Diff {
            recipe,
            inline_recipe,
            args,
            path,
        } => recipe::diff(
            registry,
            RecipeRequest {
                recipe_id: recipe.as_deref(),
                inline_recipe: inline_recipe.as_ref(),
                args: &args,
            },
            &path,
        ),
        HostCommand::ComponentScan => components::scan(registry),
        HostCommand::ComponentIncrementalScan {
            dirty_paths,
            detect_changes,
            force,
        } => components::incremental_scan(registry, dirty_paths, detect_changes, force),
        HostCommand::ComponentListInstances => components::list_instances(registry),
        HostCommand::ComponentCoverage => components::coverage(registry),
        HostCommand::ComponentGraph { level, out } => components::graph(registry, level, out),
        HostCommand::ComponentInvokeOperation {
            instance_id,
            operation,
            args,
            preview,
        } => components::invoke_operation_cmd(registry, instance_id, operation, args, preview),
        HostCommand::ComponentScaffold {
            component_id,
            args,
            preview,
        } => components::scaffold(registry, component_id, args, preview),
        HostCommand::ComponentInvokeHook {
            component_id,
            hook,
            preview,
            locator,
            instance,
            operation,
            args,
        } => components::invoke_hook_cmd(
            registry,
            component_id,
            hook,
            preview,
            locator,
            instance,
            operation,
            args,
        ),
        HostCommand::StackInvokeOperation {
            operation,
            preview,
            args,
        } => components::stack_invoke(registry, operation, preview, args),
        HostCommand::DescribeComponent { component_id } => {
            components::describe_component(registry, component_id)
        }
        HostCommand::RunTests { filter } => catalog::run_tests(registry, filter),
        HostCommand::ComponentSnapshot { path } => components::component_snapshot(registry, path),
    }
}
