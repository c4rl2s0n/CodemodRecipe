use codemod_recipe_host::protocol::dispatch::handle_command;
use codemod_recipe_host::protocol::keys as protocol_keys;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::{config::HostConfig, recipe::registry::RecipeRegistry};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

#[derive(Debug, Deserialize)]
struct RpcRequest {
    jsonrpc: String,
    id: Option<serde_json::Value>,
    method: String,
    #[serde(default)]
    params: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct RpcResponse {
    jsonrpc: &'static str,
    id: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<RpcError>,
}

#[derive(Debug, Serialize)]
struct RpcError {
    code: i64,
    message: String,
}

fn main() -> anyhow::Result<()> {
    let config = HostConfig::from_env_args();
    let mut registry =
        RecipeRegistry::new(config.workspace_root.clone(), config.codemod_root.clone());
    registry.language_registry =
        codemod_recipe_engine::LanguageRegistry::with_config(config.language_registry_config());
    registry.reload();

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(line) => line,
            Err(_) => break,
        };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(request) = serde_json::from_str::<RpcRequest>(&line) else {
            continue;
        };
        if request.jsonrpc != "2.0" {
            continue;
        }
        let Some(id) = request.id.clone() else {
            continue;
        };

        let response = handle_request(&mut registry, &request, id);
        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }

    Ok(())
}

fn handle_request(
    registry: &mut RecipeRegistry,
    request: &RpcRequest,
    id: serde_json::Value,
) -> RpcResponse {
    match request.method.as_str() {
        "initialize" => RpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(json!({
              "protocolVersion": "2024-11-05",
              "serverInfo": { "name": "codemod-mcp-rust", "version": "0.2.0" },
              "capabilities": { "tools": {} }
            })),
            error: None,
        },
        "tools/list" => RpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(json!({
              "tools": [
                {
                  "name": "list_recipes",
                  "description": "List registered recipes with schemas and diagnostics",
                  "inputSchema": { "type": "object" }
                },
                {
                  "name": "describe_recipe",
                  "description": "Describe one registered recipe",
                  "inputSchema": {
                    "type": "object",
                    "properties": { "recipe": { "type": "string" } },
                    "required": ["recipe"]
                  }
                },
                {
                  "name": "validate_recipes",
                  "description": "Reload and validate all recipes and maps (optionally one recipe by id)",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "recipe": {
                        "type": "string",
                        "description": "Optional recipe id to validate without reloading others"
                      }
                    }
                  }
                },
                {
                  "name": "preview_recipe",
                  "description": "Preview a registered or inline recipe",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "recipe": { "type": "string" },
                      "inlineRecipe": { "type": "object" },
                      "args": { "type": "object" },
                      "snippetLines": { "type": "number" }
                    }
                  }
                },
                {
                  "name": "apply_recipe",
                  "description": "Apply a previewed recipe atomically",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "recipe": { "type": "string" },
                      "inlineRecipe": { "type": "object" },
                      "args": { "type": "object" },
                      "previewToken": { "type": "string" },
                      "selection": { "type": "object" }
                    },
                    "required": ["previewToken"]
                  }
                },
                {
                  "name": "bootstrap_project",
                  "description": "Install codemod-recipe agent skills (.agents/skills/), rules (.cursor/rules/), and .codemod/ scaffolding into the workspace. Soft edit_policy recommend by default; opt into strict and/or companions.",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "force": {
                        "type": "boolean",
                        "description": "Overwrite existing files (default false)"
                      },
                      "edit_policy": {
                        "type": "string",
                        "enum": ["recommend", "strict"],
                        "description": "Rule pack: recommend (default, soft prefer+preview) or strict (recipe-first edit policy)"
                      },
                      "companions": {
                        "type": "array",
                        "items": { "type": "string", "enum": ["codebase-memory"] },
                        "description": "Optional companion rule packs (default []). Use [\"codebase-memory\"] when that MCP is configured."
                      }
                    }
                  }
                },
                {
                  "name": "dump_ast",
                  "description": "Dump the tree-sitter AST for a workspace file or inline source (Query Tools).",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "path": { "type": "string" },
                      "source": { "type": "string" },
                      "language": { "type": "string" },
                      "namedOnly": { "type": "boolean" }
                    }
                  }
                },
                {
                  "name": "debug_query",
                  "description": "Run a tree-sitter query and return match roots and captures (Query Tools).",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "path": { "type": "string" },
                      "source": { "type": "string" },
                      "language": { "type": "string" },
                      "query": { "type": "string" },
                      "instrument": { "type": "boolean" },
                      "includeSexp": { "type": "boolean" }
                    },
                    "required": ["query"]
                  }
                },
                {
                  "name": "generate_query",
                  "description": "Generate a starter tree-sitter query from a byte range in a file (Query Tools).",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "path": { "type": "string" },
                      "source": { "type": "string" },
                      "language": { "type": "string" },
                      "start": { "type": "number" },
                      "end": { "type": "number" },
                      "includeTextPredicates": { "type": "boolean" },
                      "captureLeaf": { "type": "string" },
                      "maxDepth": { "type": "number" }
                    },
                    "required": ["start"]
                  }
                },
                {
                  "name": "resolve_static_path",
                  "description": "Try to Jinja-render a path template with empty args; succeeds only when no parameters are required.",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "template": { "type": "string" }
                    },
                    "required": ["template"]
                  }
                },
                {
                  "name": "component_incremental_scan",
                  "description": "Incremental component scan: pass dirtyPaths and/or detectChanges to reparse affected roots without full workspace scan.",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "dirtyPaths": { "type": "array", "items": { "type": "string" } },
                      "detectChanges": { "type": "boolean" },
                      "force": { "type": "boolean" }
                    }
                  }
                },
                {
                  "name": "component_scan",
                  "description": "Scan stack root components: identify → parse → relate → verify, and optional toSummary for each instance. Returns instances, findings, relation edges, and the kindGraph roll-up.",
                  "inputSchema": { "type": "object" }
                },
                {
                  "name": "list_component_instances",
                  "description": "Alias of component_scan — returns instances, findings, and summaries.",
                  "inputSchema": { "type": "object" }
                },
                {
                  "name": "component_coverage",
                  "description": "Post-scan span coverage against stack.codebase globs. Unions explicit source.span ranges; whitespace excluded. Missing codebase skips with a warning. Runs scan first if cache empty. Returns per-file + aggregate ratios and gaps.",
                  "inputSchema": { "type": "object" }
                },
                {
                  "name": "component_graph",
                  "description": "Render the component relations graph as mermaid text. level: 'kinds' (default; component-level roll-up with edge counts) or 'instances' (every flat node, symbol targets as external leaves). Runs scan first if cache empty.",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "level": { "type": "string", "enum": ["kinds", "instances"] }
                    }
                  }
                },
                {
                  "name": "describe_component",
                  "description": "Component metadata: hooks present (hasIdentify/hasToSummary/hasScaffold), scaffold args, and operations.",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "componentId": { "type": "string" }
                    },
                    "required": ["componentId"]
                  }
                },
                {
                  "name": "component_dry_run_hook",
                  "description": "Dry-run a component hook (Preview WorkingTree; never writes disk). Returns result JSON + transcript (c:log visible). hook: identify|parse|verify|toSummary|scaffold|operation. Authoring agents: prefer this over component_apply_hook.",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "componentId": { "type": "string" },
                      "hook": { "type": "string" },
                      "locator": {},
                      "instance": {},
                      "operation": { "type": "string" },
                      "args": { "type": "object" }
                    },
                    "required": ["componentId", "hook"]
                  }
                },
                {
                  "name": "component_apply_hook",
                  "description": "Apply a component hook (flushes WorkingTree writes). Same args as component_dry_run_hook. Prefer dry-run first. Withhold this tool from authoring-only agent allowlists.",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "componentId": { "type": "string" },
                      "hook": { "type": "string" },
                      "locator": {},
                      "instance": {},
                      "operation": { "type": "string" },
                      "args": { "type": "object" }
                    },
                    "required": ["componentId", "hook"]
                  }
                },
                {
                  "name": "run_tests",
                  "description": "Run *.test.luau suites (fixture-based regression tests for recipes, components, and relations). Read-only for the real workspace: mutations apply to temp fixture dirs only. Optional filter matches suite id, suite target, 'suite/test', or a test name substring. Returns per-test pass/fail with error line numbers.",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "filter": { "type": "string" }
                    }
                  }
                }
              ]
            })),
            error: None,
        },
        "tools/call" => {
            let tool = request
                .params
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let arguments = request
                .params
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| json!({}));

            let result = match tool {
                "list_recipes" => handle_command(registry, HostCommand::List),
                "describe_recipe" => {
                    let recipe_id = arguments
                        .get("recipe")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    handle_command(
                        registry,
                        HostCommand::Describe {
                            recipe: recipe_id.to_string(),
                        },
                    )
                }
                "validate_recipes" => {
                    let recipe = arguments
                        .get("recipe")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    handle_command(registry, HostCommand::Validate { recipe })
                }
                "preview_recipe" => mcp_preview_or_apply(registry, &arguments, false),
                "apply_recipe" => mcp_preview_or_apply(registry, &arguments, true),
                "bootstrap_project" => {
                    let force = arguments
                        .get("force")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let edit_policy_raw = arguments
                        .get("edit_policy")
                        .and_then(|v| v.as_str())
                        .unwrap_or("recommend");
                    let companions_raw: Vec<String> = arguments
                        .get("companions")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    match (
                        codemod_recipe_host::config::bootstrap::EditPolicy::parse(edit_policy_raw),
                        codemod_recipe_host::config::bootstrap::parse_companions(&companions_raw),
                    ) {
                        (Ok(edit_policy), Ok(companions)) => {
                            codemod_recipe_host::config::bootstrap::bootstrap_project(
                                &registry.workspace_root,
                                force,
                                edit_policy,
                                &companions,
                            )
                        }
                        (Err(e), _) | (_, Err(e)) => error_json(e),
                    }
                }
                "dump_ast" => handle_command(
                    registry,
                    HostCommand::DumpAst {
                        path: arguments
                            .get("path")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        source: arguments
                            .get("source")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        language: arguments
                            .get("language")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        named_only: arguments
                            .get("namedOnly")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true),
                    },
                ),
                "debug_query" => handle_command(
                    registry,
                    HostCommand::DebugQuery {
                        path: arguments
                            .get("path")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        source: arguments
                            .get("source")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        language: arguments
                            .get("language")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        query: arguments
                            .get("query")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        instrument: arguments
                            .get("instrument")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(true),
                        include_sexp: arguments
                            .get("includeSexp")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                    },
                ),
                "generate_query" => handle_command(
                    registry,
                    HostCommand::GenerateQuery {
                        path: arguments
                            .get("path")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        source: arguments
                            .get("source")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        language: arguments
                            .get("language")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        start: arguments.get("start").and_then(|v| v.as_u64()).unwrap_or(0),
                        end: arguments.get("end").and_then(|v| v.as_u64()).unwrap_or(0),
                        include_text_predicates: arguments
                            .get("includeTextPredicates")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false),
                        capture_leaf: arguments
                            .get("captureLeaf")
                            .and_then(|v| v.as_str())
                            .map(String::from),
                        max_depth: arguments.get("maxDepth").and_then(|v| v.as_u64()),
                    },
                ),
                "resolve_static_path" => handle_command(
                    registry,
                    HostCommand::ResolveStaticPath {
                        template: arguments
                            .get("template")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    },
                ),
                "component_scan" => handle_command(registry, HostCommand::ComponentScan),
                "component_incremental_scan" => {
                    let dirty_paths = arguments
                        .get("dirtyPaths")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect()
                        })
                        .unwrap_or_default();
                    let detect_changes = arguments
                        .get("detectChanges")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    let force = arguments
                        .get("force")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    handle_command(
                        registry,
                        HostCommand::ComponentIncrementalScan {
                            dirty_paths,
                            detect_changes,
                            force,
                        },
                    )
                }
                "list_component_instances" => {
                    handle_command(registry, HostCommand::ComponentListInstances)
                }
                "component_coverage" => handle_command(registry, HostCommand::ComponentCoverage),
                "component_graph" => {
                    let level = arguments
                        .get("level")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    handle_command(registry, HostCommand::ComponentGraph { level })
                }
                "describe_component" => {
                    let component_id = arguments
                        .get("componentId")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    handle_command(registry, HostCommand::DescribeComponent { component_id })
                }
                "component_dry_run_hook" => mcp_component_invoke_hook(registry, &arguments, true),
                "component_apply_hook" => mcp_component_invoke_hook(registry, &arguments, false),
                "run_tests" => handle_command(
                    registry,
                    HostCommand::RunTests {
                        filter: arguments
                            .get("filter")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                    },
                ),
                _ => error_json(format!("Unknown tool: {tool}")),
            };

            RpcResponse {
                jsonrpc: "2.0",
                id,
                result: Some(json!({
                    "content": [{ "type": "text", "text": serde_json::to_string(&result).unwrap_or_default() }]
                })),
                error: None,
            }
        }
        _ => RpcResponse {
            jsonrpc: "2.0",
            id,
            result: None,
            error: Some(RpcError {
                code: -32601,
                message: format!("Method not found: {}", request.method),
            }),
        },
    }
}

fn mcp_component_invoke_hook(
    registry: &mut RecipeRegistry,
    arguments: &serde_json::Value,
    preview: bool,
) -> serde_json::Value {
    let component_id = arguments
        .get("componentId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if component_id.is_empty() {
        return error_json("Missing componentId");
    }
    let hook = arguments
        .get("hook")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if hook.is_empty() {
        return error_json("Missing hook");
    }
    let args = arguments
        .get("args")
        .and_then(|v| v.as_object())
        .map(|m| {
            m.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    handle_command(
        registry,
        HostCommand::ComponentInvokeHook {
            component_id,
            hook,
            preview,
            locator: arguments.get("locator").cloned(),
            instance: arguments.get("instance").cloned(),
            operation: arguments
                .get("operation")
                .and_then(|v| v.as_str())
                .map(String::from),
            args,
        },
    )
}

fn mcp_preview_or_apply(
    registry: &mut RecipeRegistry,
    arguments: &serde_json::Value,
    do_apply: bool,
) -> serde_json::Value {
    let recipe = arguments
        .get(protocol_keys::RECIPE)
        .and_then(|v| v.as_str())
        .map(String::from);
    let inline_recipe = arguments.get(protocol_keys::INLINE_RECIPE).cloned();
    if recipe.is_none() && inline_recipe.is_none() {
        return error_json("Missing recipe or inlineRecipe");
    }

    let args = json_args_to_btreemap(arguments);

    if do_apply {
        let preview_token = arguments
            .get(protocol_keys::PREVIEW_TOKEN)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let selection = arguments
            .get(protocol_keys::SELECTION)
            .cloned()
            .unwrap_or(serde_json::Value::Null);
        handle_command(
            registry,
            HostCommand::Apply {
                recipe,
                inline_recipe,
                args,
                preview_token,
                selection,
            },
        )
    } else {
        let snippet_lines = arguments
            .get(protocol_keys::SNIPPET_LINES)
            .and_then(|v| v.as_u64())
            .map(|n| n as u32);
        handle_command(
            registry,
            HostCommand::Preview {
                recipe,
                inline_recipe,
                args,
                snippet_lines,
            },
        )
    }
}

fn error_json(message: impl Into<String>) -> serde_json::Value {
    let mut value = serde_json::Map::new();
    value.insert(
        protocol_keys::OK.to_string(),
        serde_json::Value::Bool(false),
    );
    value.insert(
        protocol_keys::ERROR.to_string(),
        serde_json::Value::String(message.into()),
    );
    serde_json::Value::Object(value)
}

fn json_args_to_btreemap(arguments: &serde_json::Value) -> BTreeMap<String, serde_json::Value> {
    arguments
        .get("args")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
        .unwrap_or_default()
}
