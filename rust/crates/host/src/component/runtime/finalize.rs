//! Finalize host WorkingTree and optional post-op rescan.

use std::rc::Rc;

use crate::component::scan::{component_incremental_scan, ComponentIncrementalScanRequest};
use crate::component::types::{ComponentFinding, ComponentInstance};
use crate::edit_session::working_tree::WorkingTree;
use crate::host_api::context::{RunKind, SharedHost};
use crate::preview::diff::build_file_preview_from_change;
use crate::recipe::registry::RecipeRegistry;
use codemod_recipe_lua::json_lua::lua_to_json;
use mlua::Value;

pub(crate) fn take_transcript(host: &SharedHost) -> Option<Vec<codemod_recipe_lua::TranscriptEvent>> {
    host.lock()
        .ok()
        .map(|mut inner| std::mem::take(&mut inner.transcript))
        .filter(|t| !t.is_empty())
}

type FinalizeParts = (
    Option<Vec<String>>,
    Option<Vec<crate::protocol::FilePreview>>,
    Vec<codemod_recipe_lua::TranscriptEvent>,
);

pub(crate) fn finalize_host(host: &SharedHost, kind: RunKind) -> Result<FinalizeParts, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    let applied = if kind == RunKind::Apply {
        Some(inner.flush_to_disk()?)
    } else {
        None
    };
    let changes = std::mem::replace(&mut inner.tree, WorkingTree::new()).finalize();
    let transcript = std::mem::take(&mut inner.transcript);
    let files = if changes.is_empty() {
        None
    } else {
        Some(
            changes
                .iter()
                .filter_map(|c| build_file_preview_from_change(c, false, None).ok())
                .collect(),
        )
    };
    Ok((applied, files, transcript))
}

pub(crate) struct RescanResult {
    pub(crate) instance: Option<ComponentInstance>,
    pub(crate) findings: Option<Vec<ComponentFinding>>,
}

pub(crate) fn component_scan_after_operation(
    registry: &mut RecipeRegistry,
    instance_id: &str,
    changes: &[codemod_recipe_core::apply::file_change::FileChange],
) -> Result<RescanResult, String> {
    let dirty_paths: Vec<String> = changes.iter().map(|c| c.path().to_string()).collect();
    let scan = if dirty_paths.is_empty() {
        component_incremental_scan(
            registry,
            ComponentIncrementalScanRequest {
                detect_changes: true,
                ..Default::default()
            },
        )
    } else {
        component_incremental_scan(
            registry,
            ComponentIncrementalScanRequest {
                dirty_paths,
                ..Default::default()
            },
        )
    };
    if !scan.ok {
        return Err(scan.error.unwrap_or_else(|| "scan failed".to_string()));
    }
    let instances = scan.instances.unwrap_or_default();
    let findings = scan.findings.unwrap_or_default();
    let instance = instances.into_iter().find(|i| i.id == instance_id);
    let instance_findings: Vec<ComponentFinding> = findings
        .into_iter()
        .filter(|f| f.instance_id.as_deref() == Some(instance_id))
        .collect();
    Ok(RescanResult {
        instance,
        findings: if instance_findings.is_empty() {
            None
        } else {
            Some(instance_findings)
        },
    })
}

pub fn hook_to_json(value: &Value) -> Result<serde_json::Value, String> {
    lua_to_json(value)
}
