//! Query / AST debug HostCommand handlers.

use crate::protocol::dispatch::common::to_value;
use crate::protocol::QueryCapturesResponse;
use crate::recipe::registry::RecipeRegistry;

fn extract_capture_names(query_source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let bytes = query_source.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'@' {
            i += 1;
            let start = i;
            while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                i += 1;
            }
            if i > start {
                let name = &query_source[start..i];
                if seen.insert(name.to_string()) {
                    names.push(name.to_string());
                }
            }
            continue;
        }
        i += 1;
    }
    names
}

pub(super) fn query_captures_response(
    registry: &RecipeRegistry,
    query: Option<&str>,
    query_id: Option<&str>,
) -> QueryCapturesResponse {
    let source = if let Some(id) = query_id {
        match registry.queries_by_id().get(id) {
            Some(s) => s.clone(),
            None => {
                return QueryCapturesResponse {
                    ok: false,
                    error: Some(format!("unknown query id '{id}'")),
                    captures: None,
                };
            }
        }
    } else if let Some(q) = query {
        q.to_string()
    } else {
        return QueryCapturesResponse {
            ok: false,
            error: Some("query or queryId is required".to_string()),
            captures: None,
        };
    };
    QueryCapturesResponse {
        ok: true,
        error: None,
        captures: Some(extract_capture_names(&source)),
    }
}


pub(super) fn dump_ast(
    registry: &mut RecipeRegistry,
    path: Option<String>,
    source: Option<String>,
    language: Option<String>,
    named_only: bool,
) -> serde_json::Value {
    to_value(crate::query::query_tools::handle_dump_ast(
        registry,
        path.as_deref(),
        source.as_deref(),
        language.as_deref(),
        named_only,
    ))
}

pub(super) fn debug_query(
    registry: &mut RecipeRegistry,
    path: Option<String>,
    source: Option<String>,
    language: Option<String>,
    query: String,
    instrument: bool,
    include_sexp: bool,
) -> serde_json::Value {
    to_value(crate::query::query_tools::handle_debug_query(
        registry,
        path.as_deref(),
        source.as_deref(),
        language.as_deref(),
        &query,
        instrument,
        include_sexp,
    ))
}

pub(super) fn generate_query(
    registry: &mut RecipeRegistry,
    path: Option<String>,
    source: Option<String>,
    language: Option<String>,
    start: u64,
    end: u64,
    include_text_predicates: bool,
    capture_leaf: Option<String>,
    max_depth: Option<u64>,
) -> serde_json::Value {
    to_value(crate::query::query_tools::handle_generate_query(
        registry,
        path.as_deref(),
        source.as_deref(),
        language.as_deref(),
        start,
        end,
        include_text_predicates,
        capture_leaf.as_deref(),
        max_depth,
    ))
}

pub(super) fn resolve_static_path(template: String) -> serde_json::Value {
    to_value(crate::query::query_tools::handle_resolve_static_path(&template))
}

pub(super) fn query_captures(
    registry: &RecipeRegistry,
    query: Option<String>,
    query_id: Option<String>,
) -> serde_json::Value {
    to_value(query_captures_response(
        registry,
        query.as_deref(),
        query_id.as_deref(),
    ))
}

pub(super) fn derive_args(
    registry: &RecipeRegistry,
    recipe: String,
    source: String,
    language: Option<String>,
    path: Option<String>,
    cursor_offset: u64,
    selection_start: u64,
    selection_end: u64,
    context: std::collections::BTreeMap<String, String>,
) -> serde_json::Value {
    to_value(crate::recipe::derive_args::derive_args(
        registry,
        crate::recipe::derive_args::DeriveArgsRequest {
            recipe_id: &recipe,
            source: &source,
            language: language.as_deref(),
            path: path.as_deref(),
            cursor_offset: cursor_offset as usize,
            selection_start: selection_start as usize,
            selection_end: selection_end as usize,
            context,
        },
    ))
}

pub(super) fn filter_explorer_recipes(
    registry: &RecipeRegistry,
    path: String,
    kind: String,
) -> serde_json::Value {
    to_value(crate::explorer::filter_explorer_recipes::filter_explorer_recipes(
        registry, &path, &kind,
    ))
}

#[cfg(test)]
mod query_captures_tests {
    use super::extract_capture_names;

    #[test]
    fn extracts_unique_captures_in_order() {
        let q = r#"
(class_definition
  name: (identifier) @className
  body: (class_body (block) @body)
  (#eq? @className "X"))
"#;
        assert_eq!(
            extract_capture_names(q),
            vec!["className".to_string(), "body".to_string()]
        );
    }
}
