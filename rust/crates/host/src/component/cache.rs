use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::component::types::{context_path, ComponentFinding, ComponentInstance};
use crate::recipe::registry::RecipeRegistry;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileFingerprint {
    pub mtime_secs: u64,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RootKey {
    pub component_id: String,
    pub root_path: String,
}

#[derive(Debug, Clone)]
pub struct NewRoot {
    pub component_id: String,
    pub locator: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct RootEntry {
    pub instance_id: String,
    pub component_id: String,
    pub root_path: String,
    pub locator: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct DetectResult {
    pub dirty_paths: BTreeSet<String>,
    pub new_roots: Vec<NewRoot>,
    pub removed_roots: Vec<RootKey>,
    pub roots_to_reparse: BTreeSet<String>,
}

#[derive(Debug, Clone)]
pub struct ComponentScanCache {
    pub instances: Vec<ComponentInstance>,
    pub findings: Vec<ComponentFinding>,
    /// workspace-relative path -> root instance ids that depend on that file
    pub path_index: BTreeMap<String, BTreeSet<String>>,
    pub path_fingerprints: BTreeMap<String, FileFingerprint>,
    pub known_root_paths: BTreeMap<String, Vec<String>>,
    pub root_entries: BTreeMap<String, RootEntry>,
}

impl ComponentScanCache {
    pub fn from_scan(
        registry: &RecipeRegistry,
        instances: Vec<ComponentInstance>,
        findings: Vec<ComponentFinding>,
    ) -> Self {
        let mut cache = Self {
            instances,
            findings,
            path_index: BTreeMap::new(),
            path_fingerprints: BTreeMap::new(),
            known_root_paths: BTreeMap::new(),
            root_entries: BTreeMap::new(),
        };
        cache.rebuild_indexes(registry.workspace_root());
        cache
    }

    pub fn rebuild_indexes(&mut self, workspace_root: &Path) {
        self.path_index.clear();
        self.path_fingerprints.clear();
        self.known_root_paths.clear();
        self.root_entries.clear();

        for instance in &self.instances {
            let root_path = root_path_for_instance(instance);
            if let Some(path) = root_path.clone() {
                self.known_root_paths
                    .entry(instance.component_id.clone())
                    .or_default()
                    .push(path.clone());
                self.root_entries.insert(
                    instance.id.clone(),
                    RootEntry {
                        instance_id: instance.id.clone(),
                        component_id: instance.component_id.clone(),
                        root_path: path,
                        locator: instance.locator.clone(),
                    },
                );
            } else {
                self.root_entries.insert(
                    instance.id.clone(),
                    RootEntry {
                        instance_id: instance.id.clone(),
                        component_id: instance.component_id.clone(),
                        root_path: String::new(),
                        locator: instance.locator.clone(),
                    },
                );
            }
            index_node_paths(&instance.node, &instance.id, &mut self.path_index);
            if let Some(path) = root_path {
                self.path_index
                    .entry(path)
                    .or_default()
                    .insert(instance.id.clone());
            }
        }

        for paths in self.known_root_paths.values_mut() {
            paths.sort();
            paths.dedup();
        }

        for path in self.path_index.keys().cloned().collect::<Vec<_>>() {
            if let Some(fp) = fingerprint(workspace_root, &path) {
                self.path_fingerprints.insert(path, fp);
            }
        }
    }

    /// Fingerprint / explicit dirty paths only. Caller merges identify-diff for new/removed roots.
    pub fn detect_dirty_paths(
        &self,
        registry: &RecipeRegistry,
        explicit_dirty: &[String],
    ) -> DetectResult {
        let workspace_root = registry.workspace_root();
        let mut result = DetectResult::default();

        if !explicit_dirty.is_empty() {
            result.dirty_paths = explicit_dirty.iter().cloned().collect();
        } else {
            for (path, fp) in &self.path_fingerprints {
                match fingerprint(workspace_root, path) {
                    Some(current) if current != *fp => {
                        result.dirty_paths.insert(path.clone());
                    }
                    None => {
                        result.dirty_paths.insert(path.clone());
                    }
                    _ => {}
                }
            }
        }

        for path in &result.dirty_paths {
            if let Some(ids) = self.path_index.get(path) {
                result.roots_to_reparse.extend(ids.iter().cloned());
            }
            if let Some(entry) = self.root_entries.values().find(|e| e.root_path == *path) {
                result.roots_to_reparse.insert(entry.instance_id.clone());
            }
        }

        result
    }

    pub fn merge_rescan(
        &mut self,
        registry: &RecipeRegistry,
        removed_roots: &[RootKey],
        updated_instances: Vec<ComponentInstance>,
        updated_findings: Vec<ComponentFinding>,
    ) {
        let removed_ids: BTreeSet<String> = removed_roots
            .iter()
            .filter_map(|root| {
                self.root_entries
                    .values()
                    .find(|e| e.component_id == root.component_id && e.root_path == root.root_path)
            })
            .map(|e| e.instance_id.clone())
            .collect();

        let updated_ids: BTreeSet<String> =
            updated_instances.iter().map(|i| i.id.clone()).collect();

        self.instances
            .retain(|i| !removed_ids.contains(&i.id) && !updated_ids.contains(&i.id));
        self.findings.retain(|f| {
            f.instance_id
                .as_ref()
                .map(|id| !removed_ids.contains(id) && !updated_ids.contains(id))
                .unwrap_or(true)
        });

        self.instances.extend(updated_instances);
        self.findings.extend(updated_findings);
        self.rebuild_indexes(registry.workspace_root());
    }

    pub fn roots_for_dirty_paths(&self, dirty_paths: &[String]) -> BTreeSet<String> {
        let mut roots = BTreeSet::new();
        for path in dirty_paths {
            if let Some(ids) = self.path_index.get(path) {
                roots.extend(ids.iter().cloned());
            }
            if let Some(entry) = self.root_entries.values().find(|e| e.root_path == *path) {
                roots.insert(entry.instance_id.clone());
            }
        }
        roots
    }

    pub fn entry_for_instance(&self, instance_id: &str) -> Option<&RootEntry> {
        self.root_entries.get(instance_id)
    }
}

pub fn locator_key(locator: &serde_json::Value) -> String {
    if let Some(path) = context_path(locator) {
        format!("path:{path}")
    } else {
        locator.to_string()
    }
}

fn root_path_for_instance(instance: &ComponentInstance) -> Option<String> {
    context_path(&instance.locator)
        .map(str::to_string)
        .or_else(|| {
            instance
                .node
                .get("source")
                .and_then(|s| s.get("path"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .or_else(|| {
            instance
                .node
                .get("path")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
}

fn index_node_paths(
    value: &serde_json::Value,
    root_id: &str,
    path_index: &mut BTreeMap<String, BTreeSet<String>>,
) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(path) = map
                .get("source")
                .and_then(|s| s.get("path"))
                .and_then(|v| v.as_str())
            {
                path_index
                    .entry(path.to_string())
                    .or_default()
                    .insert(root_id.to_string());
            }
            if let Some(path) = map.get("path").and_then(|v| v.as_str()) {
                path_index
                    .entry(path.to_string())
                    .or_default()
                    .insert(root_id.to_string());
            }
            if let Some(children) = map.get("children").and_then(|v| v.as_array()) {
                for child in children {
                    index_node_paths(child, root_id, path_index);
                }
            }
            if let Some(fields) = map.get("fields").and_then(|v| v.as_array()) {
                for field in fields {
                    index_node_paths(field, root_id, path_index);
                }
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                index_node_paths(item, root_id, path_index);
            }
        }
        _ => {}
    }
}

fn fingerprint(workspace_root: &Path, relative_path: &str) -> Option<FileFingerprint> {
    let path = workspace_root.join(relative_path);
    let meta = std::fs::metadata(&path).ok()?;
    let mtime = meta.modified().ok()?;
    let mtime_secs = mtime.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs();
    Some(FileFingerprint {
        mtime_secs,
        size: meta.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_node_paths_includes_children() {
        let node = serde_json::json!({
            "source": { "path": "lib/models/a.dart" },
            "children": [
                { "source": { "path": "lib/stores/a_store.dart" } }
            ]
        });
        let mut index = BTreeMap::new();
        index_node_paths(&node, "flutter.model/a", &mut index);
        assert!(index.contains_key("lib/models/a.dart"));
        assert!(index.contains_key("lib/stores/a_store.dart"));
        assert_eq!(index.get("lib/stores/a_store.dart").unwrap().len(), 1);
    }
}
