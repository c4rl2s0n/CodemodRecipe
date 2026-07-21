use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};

static WORKSPACE_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn repo_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn temp_workspace(name: &str) -> std::path::PathBuf {
    let n = WORKSPACE_COUNTER.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("{name}_{}_{n}", std::process::id()))
}

fn setup_workspace() -> std::path::PathBuf {
    let workspace = temp_workspace("mcp");
    let recipes_dir = workspace.join(".codemod/recipes");
    std::fs::create_dir_all(&recipes_dir).unwrap();
    std::fs::copy(
        repo_root().join(".codemod/recipes/insert_log_line.yaml"),
        recipes_dir.join("insert_log_line.yaml"),
    )
    .unwrap();
    workspace
}

fn mcp_bin() -> std::path::PathBuf {
    std::env::var("CARGO_BIN_EXE_codemod_mcp")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| repo_root().join("rust/target/debug/codemod_mcp"))
}

struct McpSession {
    _child: std::process::Child,
    stdin: std::process::ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
}

impl McpSession {
    fn spawn(workspace: &std::path::Path) -> Self {
        let bin = mcp_bin();
        let mut child = Command::new(&bin)
            .env("CODEMOD_WORKSPACE_ROOT", workspace)
            .env("CODEMOD_ROOT", workspace.join(".codemod"))
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn mcp");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = child.stdout.take().expect("stdout");
        Self {
            _child: child,
            stdin,
            reader: BufReader::new(stdout),
        }
    }

    fn rpc(&mut self, id: u64, method: &str, params: serde_json::Value) -> serde_json::Value {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });
        writeln!(self.stdin, "{request}").unwrap();
        self.stdin.flush().unwrap();
        let mut line = String::new();
        self.reader.read_line(&mut line).unwrap();
        serde_json::from_str(&line).unwrap()
    }
}

#[test]
fn stdio_subprocess_lists_recipes_and_previews() {
    let workspace = setup_workspace();
    if !mcp_bin().exists() {
        eprintln!("skip: codemod_mcp binary not found");
        return;
    }

    let mut session = McpSession::spawn(&workspace);

    let init = session.rpc(
        1,
        "initialize",
        serde_json::json!({ "protocolVersion": "2024-11-05", "capabilities": {} }),
    );
    assert_eq!(init["result"]["serverInfo"]["name"], "codemod-mcp-rust");

    let tools = session.rpc(2, "tools/list", serde_json::json!({}));
    let tool_names: Vec<String> = tools["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|t| t["name"].as_str().map(str::to_string))
        .collect();
    assert!(tool_names.contains(&"list_recipes".to_string()));
    assert!(tool_names.contains(&"describe_recipe".to_string()));
    assert!(tool_names.contains(&"validate_recipes".to_string()));
    assert!(tool_names.contains(&"preview_recipe".to_string()));
    assert!(tool_names.contains(&"apply_recipe".to_string()));
    assert!(tool_names.contains(&"bootstrap_project".to_string()));

    let list = session.rpc(
        3,
        "tools/call",
        serde_json::json!({ "name": "list_recipes", "arguments": {} }),
    );
    let list_text = list["result"]["content"][0]["text"].as_str().unwrap();
    let list_json: serde_json::Value = serde_json::from_str(list_text).unwrap();
    assert_eq!(list_json["ok"], true);
    assert!(list_json["recipes"].as_array().is_some());

    let describe = session.rpc(
        3,
        "tools/call",
        serde_json::json!({
            "name": "describe_recipe",
            "arguments": { "recipe": "insert_log_line" }
        }),
    );
    let describe_text = describe["result"]["content"][0]["text"].as_str().unwrap();
    let describe_json: serde_json::Value = serde_json::from_str(describe_text).unwrap();
    assert_eq!(describe_json["ok"], true);

    let validate = session.rpc(
        4,
        "tools/call",
        serde_json::json!({ "name": "validate_recipes", "arguments": {} }),
    );
    let validate_text = validate["result"]["content"][0]["text"].as_str().unwrap();
    let validate_json: serde_json::Value = serde_json::from_str(validate_text).unwrap();
    assert_eq!(validate_json["ok"], true);

    let rel = "lib/settings.dart";
    let settings = workspace.join(rel);
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::copy(
        repo_root().join("test/fixtures/ast_paths/settings.dart"),
        &settings,
    )
    .unwrap();

    let preview = session.rpc(
        5,
        "tools/call",
        serde_json::json!({
            "name": "preview_recipe",
            "arguments": {
                "recipe": "insert_log_line",
                "args": { "file": rel }
            }
        }),
    );
    let preview_text = preview["result"]["content"][0]["text"].as_str().unwrap();
    let preview_json: serde_json::Value = serde_json::from_str(preview_text).unwrap();
    assert_eq!(
        preview_json["ok"], true,
        "preview failed: {}",
        preview_text
    );
    assert!(!preview_json["files"].as_array().unwrap().is_empty());
    let token = preview_json["previewToken"]
        .as_str()
        .expect("preview should return previewToken");

    let apply = session.rpc(
        6,
        "tools/call",
        serde_json::json!({
            "name": "apply_recipe",
            "arguments": {
                "recipe": "insert_log_line",
                "args": { "file": rel },
                "previewToken": token
            }
        }),
    );
    let apply_text = apply["result"]["content"][0]["text"].as_str().unwrap();
    let apply_json: serde_json::Value = serde_json::from_str(apply_text).unwrap();
    assert_eq!(apply_json["ok"], true, "apply failed: {apply_text}");

    let content = std::fs::read_to_string(&settings).unwrap();
    assert!(content.contains("print('codemod')"));

    let reject = session.rpc(
        7,
        "tools/call",
        serde_json::json!({
            "name": "apply_recipe",
            "arguments": {
                "recipe": "insert_log_line",
                "args": { "file": rel }
            }
        }),
    );
    let reject_text = reject["result"]["content"][0]["text"].as_str().unwrap();
    let reject_json: serde_json::Value = serde_json::from_str(reject_text).unwrap();
    assert_eq!(reject_json["ok"], false);
    assert!(
        reject_json["error"]
            .as_str()
            .unwrap()
            .contains("previewToken")
    );

    let _ = std::fs::remove_dir_all(workspace);
}

#[test]
fn stdio_subprocess_bootstraps_project() {
    let workspace = temp_workspace("mcp_bootstrap");
    std::fs::create_dir_all(&workspace).unwrap();
    if !mcp_bin().exists() {
        eprintln!("skip: codemod_mcp binary not found");
        return;
    }

    let mut session = McpSession::spawn(&workspace);

    let _ = session.rpc(
        1,
        "initialize",
        serde_json::json!({ "protocolVersion": "2024-11-05", "capabilities": {} }),
    );

    let bootstrap = session.rpc(
        2,
        "tools/call",
        serde_json::json!({
            "name": "bootstrap_project",
            "arguments": {}
        }),
    );
    let bootstrap_text = bootstrap["result"]["content"][0]["text"]
        .as_str()
        .unwrap();
    let bootstrap_json: serde_json::Value = serde_json::from_str(bootstrap_text).unwrap();
    assert_eq!(bootstrap_json["ok"], true, "bootstrap failed: {bootstrap_text}");
    assert!(
        workspace
            .join(".agents/skills/codemod-overview/SKILL.md")
            .is_file()
    );
    assert!(workspace.join(".cursor/rules/codemod-recipe.mdc").is_file());
    assert!(workspace.join(".codemod/recipes").is_dir());

    let _ = std::fs::remove_dir_all(workspace);
}
