use codemod_recipe_host::{
    config::HostConfig, dispatch, registry::RecipeRegistry, RESULT_BEGIN, RESULT_END,
};
use std::io::{self, BufRead, Write};

use codemod_recipe_host::protocol::HostCommand;

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

        let response_json = match serde_json::from_str::<HostCommand>(&line) {
            Ok(cmd) => dispatch::handle_command(&mut registry, cmd),
            Err(error) => {
                serde_json::json!({ "ok": false, "error": format!("Invalid command JSON: {error}") })
            }
        };

        writeln!(stdout, "{RESULT_BEGIN}")?;
        writeln!(stdout, "{}", serde_json::to_string(&response_json)?)?;
        writeln!(stdout, "{RESULT_END}")?;
        stdout.flush()?;
    }

    Ok(())
}
