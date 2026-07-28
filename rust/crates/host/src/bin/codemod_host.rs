use codemod_recipe_host::{
    config::HostConfig, dispatch, protocol_keys, registry::RecipeRegistry, RESULT_BEGIN, RESULT_END,
};
use std::io::{self, BufRead, Write};

use codemod_recipe_host::protocol::HostCommand;

fn main() -> anyhow::Result<()> {
    let config = HostConfig::from_env_args();
    let mut registry =
        RecipeRegistry::new(config.workspace_root.clone(), config.codemod_root.clone());
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

        let response_json = match serde_json::from_str::<HostCommand>(&line) {
            Ok(cmd) => dispatch::handle_command(&mut registry, cmd),
            Err(error) => {
                let mut value = serde_json::Map::new();
                value.insert(
                    protocol_keys::OK.to_string(),
                    serde_json::Value::Bool(false),
                );
                value.insert(
                    protocol_keys::ERROR.to_string(),
                    serde_json::Value::String(format!("Invalid command JSON: {error}")),
                );
                serde_json::Value::Object(value)
            }
        };

        writeln!(stdout, "{RESULT_BEGIN}")?;
        writeln!(stdout, "{}", serde_json::to_string(&response_json)?)?;
        writeln!(stdout, "{RESULT_END}")?;
        stdout.flush()?;
    }

    Ok(())
}
