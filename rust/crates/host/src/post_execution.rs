use codemod_recipe_yaml::model::PostExecution;
use std::collections::BTreeMap;
use std::process::Command;

use crate::template::render_string;

pub fn run_post_execution(
    actions: &[PostExecution],
    args: &BTreeMap<String, String>,
    changed_paths: &[String],
) -> Result<(), String> {
    for action in actions {
        match action {
            PostExecution::String(command) => {
                if command == "dartFormat" {
                    for path in changed_paths {
                        let rendered = render_string("dart format {{file}}", &{
                            let mut file_args = args.clone();
                            file_args.insert("file".to_string(), path.clone());
                            file_args
                        });
                        run_shell_command(&rendered)?;
                    }
                } else {
                    let rendered = render_string(command, args);
                    run_shell_command(&rendered)?;
                }
            }
            PostExecution::Map(_) => {}
        }
    }
    Ok(())
}

fn run_shell_command(command: &str) -> Result<(), String> {
    let status = Command::new("sh")
        .arg("-c")
        .arg(command)
        .status()
        .map_err(|e| format!("Failed to run postExecution `{command}`: {e}"))?;
    if !status.success() {
        return Err(format!("postExecution failed (exit={status}): {command}"));
    }
    Ok(())
}
