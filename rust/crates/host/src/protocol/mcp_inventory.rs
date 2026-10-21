//! Single MCP tool inventory — SSOT for tools/list metadata and HostCommand mapping.
//! tools/call argument extraction stays in `codemod_mcp.rs`; names must match this list.

use serde_json::{json, Value};

#[derive(Debug, Clone, Copy)]
pub struct McpToolMeta {
    pub name: &'static str,
    pub description: &'static str,
    /// HostCommand variant ident, or None for protocol-free (DIRECT) tools.
    pub host_command: Option<&'static str>,
}

pub fn tool_metas() -> &'static [McpToolMeta] {
    &[
        McpToolMeta {
            name: "list_recipes",
            description: "List registered recipes with schemas and diagnostics",
            host_command: Some("List"),
        },
        McpToolMeta {
            name: "describe_recipe",
            description: "Describe one registered recipe",
            host_command: Some("Describe"),
        },
        McpToolMeta {
            name: "validate_recipes",
            description: "Reload and validate all recipes and maps (optionally one recipe by id)",
            host_command: Some("Validate"),
        },
        McpToolMeta {
            name: "preview_recipe",
            description: "Preview a registered or inline recipe. Returns previewToken, files, transcript, and top-level logs[] (c:log details).",
            host_command: Some("Preview"),
        },
        McpToolMeta {
            name: "apply_recipe",
            description: "Apply a previewed recipe atomically",
            host_command: Some("Apply"),
        },
        McpToolMeta {
            name: "bootstrap_project",
            description: "Install codemod-recipe agent skills (.agents/skills/), rules (.cursor/rules/), and .codemod/ scaffolding into the workspace. Soft edit_policy recommend by default; opt into strict and/or companions.",
            host_command: None,
        },
        McpToolMeta {
            name: "dump_ast",
            description: "Dump the tree-sitter AST for a workspace file or inline source (Query Tools). Call before authoring queries. Pass namedOnly:false to include keywords and punctuation (anonymous tokens).",
            host_command: Some("DumpAst"),
        },
        McpToolMeta {
            name: "debug_query",
            description: "Run a tree-sitter query and return match roots and captures (Query Tools). Optional instrument (default true) injects @__layer_* for depth visualization; pass instrument:false if debugging instrumentation itself. Dump AST first (dump_ast, namedOnly:false for keywords) before authoring queries.",
            host_command: Some("DebugQuery"),
        },
        McpToolMeta {
            name: "generate_query",
            description: "Generate a starter tree-sitter query from a byte range in a file (Query Tools).",
            host_command: Some("GenerateQuery"),
        },
        McpToolMeta {
            name: "resolve_static_path",
            description: "Try to Jinja-render a path template with empty args; succeeds only when no parameters are required.",
            host_command: Some("ResolveStaticPath"),
        },
        McpToolMeta {
            name: "component_incremental_scan",
            description: "Incremental component scan: pass dirtyPaths and/or detectChanges to reparse affected roots without full workspace scan.",
            host_command: Some("ComponentIncrementalScan"),
        },
        McpToolMeta {
            name: "component_scan",
            description: "Scan stack root components: identify \u{2192} parse \u{2192} relate \u{2192} verify, and optional toSummary for each instance. Returns summary (architecture ratio, dead composers, top product gaps, finding counts) plus instances, findings, relation edges, and kindGraph. Prefer reading summary first.",
            host_command: Some("ComponentScan"),
        },
        McpToolMeta {
            name: "list_component_instances",
            description: "Alias of component_scan \u{2014} returns instances, findings, and summary.",
            host_command: Some("ComponentListInstances"),
        },
        McpToolMeta {
            name: "component_coverage",
            description: "Post-scan span coverage against stack.codebase globs. Unions explicit source.span ranges; whitespace excluded. Missing codebase skips with a warning. Runs scan first if cache empty. Returns per-file + aggregate ratios and gaps.",
            host_command: Some("ComponentCoverage"),
        },
        McpToolMeta {
            name: "component_graph",
            description: "Render the component relations graph as mermaid text. level: 'kinds' (default; component-level roll-up with edge counts) or 'instances' (every flat node, symbol targets as external leaves). Optional out: workspace-relative path to write the mermaid file. Runs scan first if cache empty.",
            host_command: Some("ComponentGraph"),
        },
        McpToolMeta {
            name: "component_snapshot",
            description: "Write a deterministic component scan snapshot JSON (meta + instances, edges, kindGraph, findings; stable ordering) under the codemod root. Default path: .cache/scan-snapshot.json. Optional path is codemod-root-relative; rejects traversal outside the codemod root. Rewrites only when non-meta content changes. component_scan also updates the default snapshot.",
            host_command: Some("ComponentSnapshot"),
        },
        McpToolMeta {
            name: "describe_component",
            description: "Component metadata: hooks present (hasIdentify/hasToSummary/hasScaffold), scaffold args, and operations.",
            host_command: Some("DescribeComponent"),
        },
        McpToolMeta {
            name: "component_dry_run_hook",
            description: "Dry-run a component hook (Preview WorkingTree; never writes disk). Returns result JSON + transcript + top-level logs[] (c:log details) + previewToken. component_scan has no logs — use this to debug parse/identify. hook: identify|parse|verify|toSummary|scaffold|operation. Authoring agents: prefer this over component_apply_hook.",
            host_command: Some("ComponentInvokeHook"),
        },
        McpToolMeta {
            name: "component_apply_hook",
            description: "Apply a component hook (flushes WorkingTree writes). Requires previewToken from component_dry_run_hook with the same args. Withhold this tool from authoring-only agent allowlists.",
            host_command: Some("ComponentInvokeHook"),
        },
        McpToolMeta {
            name: "stack_dry_run_operation",
            description: "Dry-run a stack-level operation (Preview WorkingTree; never writes disk). Returns previewToken. Stack ops are instance-less fan-outs (e.g. createCommand).",
            host_command: Some("StackInvokeOperation"),
        },
        McpToolMeta {
            name: "stack_apply_operation",
            description: "Apply a stack-level operation (flushes WorkingTree writes). Requires previewToken from stack_dry_run_operation with the same args. Withhold from authoring-only agent allowlists.",
            host_command: Some("StackInvokeOperation"),
        },
        McpToolMeta {
            name: "run_tests",
            description: "Run *.test.luau suites (fixture-based regression tests for recipes, components, and relations). Read-only for the real workspace: mutations apply to temp fixture dirs only. Optional filter matches suite id, suite target, 'suite/test', or a test name substring. Returns per-test pass/fail with error line numbers.",
            host_command: Some("RunTests"),
        },
    ]
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
        "preview_recipe" => {
            json!({"type": "object", "properties": {"recipe": {"type": "string"}, "inlineRecipe": {"type": "string", "description": "Luau source string that returns a recipe table"}, "args": {"type": "object"}, "snippetLines": {"type": "number"}}})
        }
        "apply_recipe" => {
            json!({"type": "object", "properties": {"recipe": {"type": "string"}, "inlineRecipe": {"type": "string", "description": "Luau source string that returns a recipe table"}, "args": {"type": "object"}, "previewToken": {"type": "string"}, "selection": {"type": "object"}}, "required": ["previewToken"]})
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
