//! Finalize host WorkingTree and optional post-op rescan.

use crate::component::entry;
use crate::component::ComponentHost;
use crate::component::scan::{component_incremental_scan, ComponentIncrementalScanRequest};
use crate::component::types::{ComponentFinding, ComponentInstance};
use crate::host_api::context::{RunKind, SharedHost};
use crate::preview::diff::build_file_preview_from_change;
use crate::recipe::registry::RecipeRegistry;
use codemod_recipe_core::apply::file_change::FileChange;
use codemod_recipe_lua::json_lua::lua_to_json;
use mlua::Value;

pub(crate) fn take_transcript(host: &SharedHost) -> Option<Vec<codemod_recipe_lua::TranscriptEvent>> {
    entry::with_component_host(host, |h| Ok(h.take_transcript()))
        .ok()
        .filter(|t| !t.is_empty())
}

pub(crate) struct FinalizeParts {
    pub applied: Option<Vec<String>>,
    pub changes: Vec<FileChange>,
    pub files: Option<Vec<crate::component::FilePreview>>,
    pub transcript: Vec<codemod_recipe_lua::TranscriptEvent>,
}

/// Finalize via [`ComponentHost`] (flush on Apply, drain tree + transcript).
pub(crate) fn finalize_via_facade(
    host: &mut dyn ComponentHost,
    kind: RunKind,
) -> Result<FinalizeParts, String> {
    let applied = if kind == RunKind::Apply {
        Some(host.flush_to_disk()?)
    } else {
        None
    };
    let changes = host.finalize_tree();
    let transcript = host.take_transcript();
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
    Ok(FinalizeParts {
        applied,
        changes,
        files,
        transcript,
    })
}

pub(crate) fn finalize_host(host: &SharedHost, kind: RunKind) -> Result<FinalizeParts, String> {
    entry::with_component_host(host, |h| finalize_via_facade(h, kind))
}

pub(crate) struct RescanResult {
    pub(crate) instance: Option<ComponentInstance>,
    pub(crate) findings: Option<Vec<ComponentFinding>>,
}

pub(crate) fn component_scan_after_operation(
    registry: &mut RecipeRegistry,
    instance_id: &str,
    changes: &[FileChange],
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
