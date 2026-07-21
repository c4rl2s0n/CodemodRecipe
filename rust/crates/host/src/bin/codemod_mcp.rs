use codemod_recipe_host::dispatch::handle_command;
use codemod_recipe_host::protocol::HostCommand;
use codemod_recipe_host::{config::HostConfig, registry::RecipeRegistry};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::io::{self, BufRead, Write};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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
    let mut registry = RecipeRegistry::new(config.workspace_root.clone(), config.codemod_root.clone());
    registry.language_config = config.language_registry_config();
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
                  "description": "Install codemod-recipe agent skills (.agents/skills/), rules (.cursor/rules/), and .codemod/ scaffolding into the workspace",
                  "inputSchema": {
                    "type": "object",
                    "properties": {
                      "force": {
                        "type": "boolean",
                        "description": "Overwrite existing files (default false)"
                      }
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
                    handle_command(
                        registry,
                        HostCommand::Validate { recipe },
                    )
                }
                "preview_recipe" => mcp_preview_or_apply(registry, &arguments, false),
                "apply_recipe" => mcp_preview_or_apply(registry, &arguments, true),
                "bootstrap_project" => {
                    let force = arguments
                        .get("force")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    codemod_recipe_host::bootstrap::bootstrap_project(
                        &registry.workspace_root,
                        force,
                    )
                }
                _ => json!({ "ok": false, "error": format!("Unknown tool: {tool}") }),
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

fn mcp_preview_or_apply(
    registry: &mut RecipeRegistry,
    arguments: &serde_json::Value,
    do_apply: bool,
) -> serde_json::Value {
    let recipe = arguments
        .get("recipe")
        .and_then(|v| v.as_str())
        .map(String::from);
    let inline_recipe = arguments.get("inlineRecipe").cloned();
    if recipe.is_none() && inline_recipe.is_none() {
        return json!({ "ok": false, "error": "Missing recipe or inlineRecipe" });
    }

    let args = json_args_to_btreemap(arguments);

    if do_apply {
        let preview_token = arguments
            .get("previewToken")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let selection = arguments
            .get("selection")
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
            .get("snippetLines")
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

fn json_args_to_btreemap(arguments: &serde_json::Value) -> BTreeMap<String, String> {
    arguments
        .get("args")
        .and_then(|v| v.as_object())
        .map(|obj| {
            obj.iter()
                .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                .collect()
        })
        .unwrap_or_default()
}
