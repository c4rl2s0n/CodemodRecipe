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

    // One-shot CLI: print component_scan.summary JSON and exit (D-029).
    if std::env::args().any(|a| a == "--scan-summary") {
        let scan = handle_command(&mut registry, HostCommand::ComponentScan);
        let summary = scan
            .get("summary")
            .cloned()
            .unwrap_or_else(|| json!({ "ok": scan.get("ok"), "error": scan.get("error") }));
        println!("{}", serde_json::to_string_pretty(&summary)?);
        return Ok(());
    }

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
              "tools": codemod_recipe_host::protocol::mcp_inventory::tools_array(),
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
                    let out = arguments
                        .get("out")
                        .and_then(|v| v.as_str())
                        .map(String::from);
                    handle_command(registry, HostCommand::ComponentGraph { level, out })
                }
                "component_snapshot" => {
                    let path = arguments
                        .get("path")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(String::from);
                    handle_command(registry, HostCommand::ComponentSnapshot { path })
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
                "stack_dry_run_operation" => mcp_stack_invoke_operation(registry, &arguments, true),
                "stack_apply_operation" => mcp_stack_invoke_operation(registry, &arguments, false),
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

fn mcp_stack_invoke_operation(
    registry: &mut RecipeRegistry,
    arguments: &serde_json::Value,
    preview: bool,
) -> serde_json::Value {
    let operation = arguments
        .get("operation")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if operation.is_empty() {
        return error_json("Missing operation");
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
        HostCommand::StackInvokeOperation {
            operation,
            preview,
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
