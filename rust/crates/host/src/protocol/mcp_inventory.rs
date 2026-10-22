//! MCP tool inventory — hand `input_schema`; tool_metas from generated `mcp_inventory.g.rs`.
//! tools/call argument extraction stays in `codemod_mcp.rs`; names must match generated metas.
//! Regenerate metas: component operation `repo.host_protocol` / `syncAdapters`.

mod mcp_inventory_generated {
    include!("mcp_inventory.g.rs");
}

use serde_json::{json, Value};

#[derive(Debug, Clone, Copy)]
pub struct McpToolMeta {
    pub name: &'static str,
    pub description: &'static str,
    /// HostCommand variant ident, or None for protocol-free (DIRECT) tools.
    pub host_command: Option<&'static str>,
}

pub fn tool_metas() -> &'static [McpToolMeta] {
    mcp_inventory_generated::generated_tool_metas()
}

pub fn tool_names() -> Vec<&'static str> {
    tool_metas().iter().map(|t| t.name).collect()
}

fn input_schema(name: &str) -> Value {
    match name {
        "list_recipes" => json!({"type": "object"}),
        "describe_recipe" => {
            json!({"type": "object", "properties": {"recipe": {"type": "string"}}, "required": ["recipe"]})
        }
        "validate_recipes" => {
            json!({"type": "object", "properties": {"recipe": {"type": "string", "description": "Optional recipe id to validate without reloading others"}}})
        }
        "reload_recipes" => json!({"type": "object"}),
        "preview_recipe" => {
            json!({"type": "object", "properties": {"recipe": {"type": "string"}, "inlineRecipe": {"type": "string", "description": "Luau source string that returns a recipe table"}, "args": {"type": "object"}, "snippetLines": {"type": "number"}}})
        }
        "diff_recipe" => {
            json!({"type": "object", "properties": {"recipe": {"type": "string"}, "inlineRecipe": {"type": "string", "description": "Luau source string that returns a recipe table"}, "args": {"type": "object"}, "path": {"type": "string"}}, "required": ["path"]})
        }
        "apply_recipe" => {
            json!({"type": "object", "properties": {"recipe": {"type": "string"}, "inlineRecipe": {"type": "string", "description": "Luau source string that returns a recipe table"}, "args": {"type": "object"}, "previewToken": {"type": "string"}, "selection": {"type": "object"}}, "required": ["previewToken"]})
        }
        "derive_args" => {
            json!({"type": "object", "properties": {"recipe": {"type": "string"}, "source": {"type": "string"}, "language": {"type": "string"}, "path": {"type": "string"}, "cursorOffset": {"type": "number"}, "selectionStart": {"type": "number"}, "selectionEnd": {"type": "number"}, "context": {"type": "object", "additionalProperties": {"type": "string"}}}, "required": ["recipe", "source"]})
        }
        "filter_explorer_recipes" => {
            json!({"type": "object", "properties": {"path": {"type": "string"}, "kind": {"type": "string", "enum": ["file", "folder"]}}, "required": ["path", "kind"]})
        }
        "bootstrap_project" => {
            json!({"type": "object", "properties": {"force": {"type": "boolean", "description": "Overwrite existing files (default false)"}, "edit_policy": {"type": "string", "enum": ["recommend", "strict"], "description": "Rule pack: recommend (default, soft prefer+preview) or strict (recipe-first edit policy)"}, "companions": {"type": "array", "items": {"type": "string", "enum": ["codebase-memory"]}, "description": "Optional companion rule packs (default []). Use [\"codebase-memory\"] when that MCP is configured."}}})
        }
        "dump_ast" => {
            json!({"type": "object", "properties": {"path": {"type": "string"}, "source": {"type": "string"}, "language": {"type": "string"}, "namedOnly": {"type": "boolean"}}})
        }
        "debug_query" => {
            json!({"type": "object", "properties": {"path": {"type": "string"}, "source": {"type": "string"}, "language": {"type": "string"}, "query": {"type": "string"}, "instrument": {"type": "boolean"}, "includeSexp": {"type": "boolean"}}, "required": ["query"]})
        }
        "generate_query" => {
            json!({"type": "object", "properties": {"path": {"type": "string"}, "source": {"type": "string"}, "language": {"type": "string"}, "start": {"type": "number"}, "end": {"type": "number"}, "includeTextPredicates": {"type": "boolean"}, "captureLeaf": {"type": "string"}, "maxDepth": {"type": "number"}}, "required": ["start"]})
        }
        "query_captures" => {
            json!({"type": "object", "properties": {"query": {"type": "string"}, "queryId": {"type": "string"}}})
        }
        "resolve_static_path" => {
            json!({"type": "object", "properties": {"template": {"type": "string"}}, "required": ["template"]})
        }
        "component_incremental_scan" => {
            json!({"type": "object", "properties": {"dirtyPaths": {"type": "array", "items": {"type": "string"}}, "detectChanges": {"type": "boolean"}, "force": {"type": "boolean"}}})
        }
        "component_scan" => json!({"type": "object"}),
        "list_component_instances" => json!({"type": "object"}),
        "component_coverage" => json!({"type": "object"}),
        "component_graph" => {
            json!({"type": "object", "properties": {"level": {"type": "string", "enum": ["kinds", "instances"]}, "out": {"type": "string"}}})
        }
        "component_snapshot" => {
            json!({"type": "object", "properties": {"path": {"type": "string"}}})
        }
        "describe_component" => {
            json!({"type": "object", "properties": {"componentId": {"type": "string"}}, "required": ["componentId"]})
        }
        "component_dry_run_hook" => {
            json!({"type": "object", "properties": {"componentId": {"type": "string"}, "hook": {"type": "string"}, "locator": {}, "instance": {}, "operation": {"type": "string"}, "args": {"type": "object"}}, "required": ["componentId", "hook"]})
        }
        "component_apply_hook" => {
            json!({"type": "object", "properties": {"componentId": {"type": "string"}, "hook": {"type": "string"}, "locator": {}, "instance": {}, "operation": {"type": "string"}, "args": {"type": "object"}, "previewToken": {"type": "string"}}, "required": ["componentId", "hook", "previewToken"]})
        }
        "stack_dry_run_operation" => {
            json!({"type": "object", "properties": {"operation": {"type": "string"}, "args": {"type": "object"}}, "required": ["operation"]})
        }
        "stack_apply_operation" => {
            json!({"type": "object", "properties": {"operation": {"type": "string"}, "args": {"type": "object"}, "previewToken": {"type": "string"}}, "required": ["operation", "previewToken"]})
        }
        "run_tests" => json!({"type": "object", "properties": {"filter": {"type": "string"}}}),
        _ => json!({ "type": "object" }),
    }
}

/// MCP `tools/list` tools array (derived from [`tool_metas`]).
pub fn tools_array() -> Vec<Value> {
    tool_metas()
        .iter()
        .map(|t| {
            json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": input_schema(t.name),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tool_names_are_unique() {
        let mut names = tool_names();
        let before = names.len();
        names.sort();
        names.dedup();
        assert_eq!(before, names.len());
    }
}
