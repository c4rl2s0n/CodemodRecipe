//! Snapshot file write helpers.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::sandbox::path_sandbox::PathSandbox;

pub(crate) fn write_text_if_changed(codemod_root: &Path, relative: &str, body: &str) -> Result<bool, String> {
    let abs = resolve_codemod_relative(codemod_root, relative)?;
    if let Some(parent) = abs.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if abs.exists() {
        let existing = std::fs::read_to_string(&abs).map_err(|e| e.to_string())?;
        if existing == body {
            return Ok(false);
        }
    }
    std::fs::write(&abs, body).map_err(|e| e.to_string())?;
    Ok(true)
}

pub(crate) fn resolve_codemod_relative(codemod_root: &Path, relative: &str) -> Result<PathBuf, String> {
    PathSandbox::new(codemod_root.to_path_buf())
        .resolve_workspace_relative(relative)
        .map_err(|e| e.message)
}

pub(crate) fn should_write_json(abs: &Path, new_content: &serde_json::Value) -> Result<bool, String> {
    if !abs.exists() {
        return Ok(true);
    }
    let existing = std::fs::read_to_string(abs).map_err(|e| e.to_string())?;
    let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&existing) else {
        return Ok(true);
    };
    let mut old_content = serde_json::json!({
        "instances": parsed.get("instances").cloned().unwrap_or(serde_json::json!([])),
        "edges": parsed.get("edges").cloned().unwrap_or(serde_json::json!([])),
        "kindGraph": parsed.get("kindGraph").cloned().unwrap_or(serde_json::json!([])),
        "findings": parsed.get("findings").cloned().unwrap_or(serde_json::json!([])),
        "relations": parsed.get("relations").cloned().unwrap_or(serde_json::json!({})),
    });
    if let Some(cov) = parsed.get("coverage") {
        old_content["coverage"] = cov.clone();
    }
    let old_s = serde_json::to_string(&old_content).map_err(|e| e.to_string())?;
    let new_s = serde_json::to_string(new_content).map_err(|e| e.to_string())?;
    Ok(old_s != new_s)
}

pub(crate) fn unix_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
