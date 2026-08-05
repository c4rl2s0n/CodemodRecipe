//! Host handlers for Query Tools (dump_ast / debug_query / generate_query).

use std::path::Path;

use codemod_recipe_engine::engine::Engine;
use codemod_recipe_query_tools::{
    debug_query, dump_ast, generate_query, DebugOptions, DumpOptions, GenerateOptions,
};

use crate::path_sandbox::PathSandbox;
use crate::protocol::{
    DebugQueryResponse, DumpAstResponse, GenerateQueryResponse, ResolveStaticPathResponse,
};
use crate::registry::RecipeRegistry;
use crate::template::try_resolve_static_template;

fn resolve_source(
    registry: &RecipeRegistry,
    path: Option<&str>,
    source: Option<&str>,
) -> Result<(String, String), String> {
    if let Some(src) = source {
        let hint = path.unwrap_or("buffer.dart").to_string();
        return Ok((src.to_string(), hint));
    }
    let relative = path.ok_or_else(|| "path or source is required".to_string())?;
    let sandbox = PathSandbox::new(registry.workspace_root.clone());
    let absolute = sandbox
        .resolve_workspace_relative(relative)
        .map_err(|e| e.to_string())?;
    let text = std::fs::read_to_string(&absolute)
        .map_err(|e| format!("Failed to read {relative}: {e}"))?;
    Ok((text, relative.to_string()))
}

fn resolve_engine<'a>(
    registry: &'a mut RecipeRegistry,
    language: Option<&str>,
    path_hint: &str,
) -> Result<&'a mut Engine, String> {
    registry
        .language_registry()
        .resolve_for_edit(language, path_hint)
        .map_err(|e| e.to_string())
}

fn cached_parse(
    registry: &mut RecipeRegistry,
    path_key: &str,
    source: &str,
    language: Option<&str>,
    path_hint: &str,
) -> Result<crate::ast_cache::CachedTree, String> {
    let engine = resolve_engine(registry, language, path_hint)?;
    let lang = engine.language();
    registry
        .ast_cache
        .get_or_parse(path_key, source, &lang)
}

pub fn handle_dump_ast(
    registry: &mut RecipeRegistry,
    path: Option<&str>,
    source: Option<&str>,
    language: Option<&str>,
    named_only: bool,
) -> DumpAstResponse {
    match dump_ast_inner(registry, path, source, language, named_only) {
        Ok((has_error, root)) => DumpAstResponse {
            ok: true,
            error: None,
            has_error: Some(has_error),
            root: Some(root),
        },
        Err(e) => DumpAstResponse {
            ok: false,
            error: Some(e),
            has_error: None,
            root: None,
        },
    }
}

fn dump_ast_inner(
    registry: &mut RecipeRegistry,
    path: Option<&str>,
    source: Option<&str>,
    language: Option<&str>,
    named_only: bool,
) -> Result<(bool, serde_json::Value), String> {
    let (text, hint) = resolve_source(registry, path, source)?;
    let path_key = path.unwrap_or(&hint);
    let cached = cached_parse(registry, path_key, &text, language, &hint)?;
    let has_error = cached.tree.root_node().has_error();
    let opts = DumpOptions {
        named_only,
        ..DumpOptions::default()
    };
    let root = dump_ast(cached.tree.root_node(), &cached.source, &opts);
    let value = serde_json::to_value(root).map_err(|e| e.to_string())?;
    Ok((has_error, value))
}

pub fn handle_debug_query(
    registry: &mut RecipeRegistry,
    path: Option<&str>,
    source: Option<&str>,
    language: Option<&str>,
    query: &str,
    instrument: bool,
    include_sexp: bool,
) -> DebugQueryResponse {
    match debug_query_inner(
        registry,
        path,
        source,
        language,
        query,
        instrument,
        include_sexp,
    ) {
        Ok(result) => DebugQueryResponse {
            ok: true,
            error: None,
            result: Some(result),
        },
        Err(e) => DebugQueryResponse {
            ok: false,
            error: Some(e),
            result: None,
        },
    }
}

fn debug_query_inner(
    registry: &mut RecipeRegistry,
    path: Option<&str>,
    source: Option<&str>,
    language: Option<&str>,
    query: &str,
    instrument: bool,
    include_sexp: bool,
) -> Result<serde_json::Value, String> {
    let (text, hint) = resolve_source(registry, path, source)?;
    let path_key = path.unwrap_or(&hint);
    let engine = resolve_engine(registry, language, &hint)?;
    let language = engine.language();
    let cached = registry.ast_cache.get_or_parse(path_key, &text, &language)?;
    let opts = DebugOptions {
        instrument,
        include_sexp,
        ..DebugOptions::default()
    };
    let result = debug_query(&language, &cached.source, query, &opts).map_err(|e| e.to_string())?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}

pub fn handle_generate_query(
    registry: &mut RecipeRegistry,
    path: Option<&str>,
    source: Option<&str>,
    language: Option<&str>,
    start: u64,
    end: u64,
    include_text_predicates: bool,
    capture_leaf: Option<&str>,
    max_depth: Option<u64>,
) -> GenerateQueryResponse {
    match generate_query_inner(
        registry,
        path,
        source,
        language,
        start as usize,
        end as usize,
        include_text_predicates,
        capture_leaf,
        max_depth.map(|d| d as usize),
    ) {
        Ok(gen) => GenerateQueryResponse {
            ok: true,
            error: None,
            query: Some(gen.query),
            capture_suggestion: Some(gen.capture_suggestion),
        },
        Err(e) => GenerateQueryResponse {
            ok: false,
            error: Some(e),
            query: None,
            capture_suggestion: None,
        },
    }
}

fn generate_query_inner(
    registry: &mut RecipeRegistry,
    path: Option<&str>,
    source: Option<&str>,
    language: Option<&str>,
    start: usize,
    end: usize,
    include_text_predicates: bool,
    capture_leaf: Option<&str>,
    max_depth: Option<usize>,
) -> Result<codemod_recipe_query_tools::GeneratedQuery, String> {
    let (text, hint) = resolve_source(registry, path, source)?;
    let path_key = path.unwrap_or(&hint);
    let engine = resolve_engine(registry, language, &hint)?;
    let language = engine.language();
    let cached = registry.ast_cache.get_or_parse(path_key, &text, &language)?;
    let opts = GenerateOptions {
        include_text_predicates,
        capture_leaf: capture_leaf.unwrap_or("target").to_string(),
        max_depth,
    };
    generate_query(&cached.tree, &cached.source, start, end, &opts).map_err(|e| e.to_string())
}

pub fn handle_resolve_static_path(template: &str) -> ResolveStaticPathResponse {
    match try_resolve_static_template(template) {
        Ok(path) => ResolveStaticPathResponse {
            ok: true,
            error: None,
            path: Some(path),
            static_resolvable: true,
        },
        Err(e) => ResolveStaticPathResponse {
            ok: true,
            error: Some(e),
            path: None,
            static_resolvable: false,
        },
    }
}

/// True when `relative` exists under the workspace root as a file.
#[allow(dead_code)]
pub fn workspace_file_exists(registry: &RecipeRegistry, relative: &str) -> bool {
    let sandbox = PathSandbox::new(registry.workspace_root.clone());
    sandbox
        .resolve_workspace_relative(relative)
        .ok()
        .map(|p| Path::new(&p).is_file())
        .unwrap_or(false)
}
