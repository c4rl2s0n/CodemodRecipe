use codemod_recipe_host::{config::HostConfig, registry::RecipeRegistry, runner};
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
    let mut registry = RecipeRegistry::new(config.workspace_root, config.codemod_root);
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
              "serverInfo": { "name": "codemod-mcp-rust", "version": "0.1.0" },
              "capabilities": { "tools": {} }
            })),
            error: None,
        },
        "tools/list" => RpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(json!({
              "tools": [
                { "name": "list_recipes", "description": "List registered recipes", "inputSchema": { "type": "object" } },
                { "name": "preview_recipe", "description": "Preview a recipe against args", "inputSchema": { "type": "object", "properties": { "recipe": { "type": "string" }, "args": { "type": "object" } }, "required": ["recipe","args"] } },
                { "name": "apply_recipe", "description": "Apply a recipe (unsafe: writes files)", "inputSchema": { "type": "object", "properties": { "recipe": { "type": "string" }, "args": { "type": "object" } }, "required": ["recipe","args"] } }
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
                "list_recipes" => json!({ "ok": true, "recipes": registry.list_ids() }),
                "preview_recipe" => {
                    let recipe_id = arguments
                        .get("recipe")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let args = json_args_to_btreemap(&arguments);
                    preview_or_apply(registry, recipe_id, &args, false)
                }
                "apply_recipe" => {
                    let recipe_id = arguments
                        .get("recipe")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let args = json_args_to_btreemap(&arguments);
                    preview_or_apply(registry, recipe_id, &args, true)
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

fn preview_or_apply(
    registry: &RecipeRegistry,
    recipe_id: &str,
    args: &BTreeMap<String, String>,
    do_apply: bool,
) -> serde_json::Value {
    match runner::run_recipe_on_file(registry, recipe_id, args) {
        Ok((file, before, result)) => {
            if do_apply {
                let file_path = registry.resolve_file_path(&file);
                if let Err(error) = runner::write_file(&file_path, &result.modified) {
                    return json!({ "ok": false, "error": error });
                }
                json!({ "ok": true, "applied": [file] })
            } else {
                json!({
                    "ok": true,
                    "files": [{ "path": file, "original": before, "modified": result.modified }]
                })
            }
        }
        Err(error) => json!({ "ok": false, "error": error }),
    }
}
