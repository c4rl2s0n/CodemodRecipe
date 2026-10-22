//! Component-facing host facade (D-025 F).
//!
//! Runtime / delegate code talks to this trait instead of locking `SharedHost`
//! and peeking `HostInner` fields. The concrete Luau session still uses
//! `SharedHost` at the entry adapter (the host entry adapter); production
//! [`HostInner`] implements this trait so finalize / stack / graph stay
//! independently testable via [`FakeComponentHost`].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use codemod_recipe_core::apply::file_change::FileChange;
use codemod_recipe_lua::model::{ComponentDef, TranscriptEvent};


/// Narrow API the component subsystem needs from a running host session.
///
/// No protocol / MCP / VS Code types. Prefer returning [`FileChange`] over
/// exposing [`WorkingTree`] across a crate boundary.
pub trait ComponentHost {
    fn codemod_root(&self) -> &Path;
    fn components(&self) -> &BTreeMap<String, ComponentDef>;
    fn component_stack(&self) -> &[String];
    fn push_component(&mut self, id: &str) -> Result<(), String>;
    fn pop_component(&mut self, id: &str);
    fn graph_flat_json(&self) -> Option<&serde_json::Value>;
    fn graph_edges_json(&self) -> Option<&serde_json::Value>;
    fn set_graph_json(&mut self, flat: serde_json::Value, edges: serde_json::Value);
    /// Flush staged edits to disk (Apply runs only).
    fn flush_to_disk(&self) -> Result<Vec<String>, String>;
    /// Take ownership of the working-tree change list and reset the tree.
    fn finalize_tree(&mut self) -> Vec<FileChange>;
    /// Drain transcript events (empty → `None` at call sites that filter).
    fn take_transcript(&mut self) -> Vec<TranscriptEvent>;
}

/// In-memory [`ComponentHost`] for unit tests (no WorkingTree / protocol / MCP).
#[derive(Debug, Default)]
pub struct FakeComponentHost {
    pub codemod_root: PathBuf,
    pub components: BTreeMap<String, ComponentDef>,
    pub component_stack: Vec<String>,
    pub graph_flat_json: Option<serde_json::Value>,
    pub graph_edges_json: Option<serde_json::Value>,
    pub pending_changes: Vec<FileChange>,
    pub transcript: Vec<TranscriptEvent>,
    pub flush_paths: Vec<String>,
    pub flush_error: Option<String>,
}

impl FakeComponentHost {
    pub fn new(codemod_root: impl Into<PathBuf>) -> Self {
        Self {
            codemod_root: codemod_root.into(),
            ..Self::default()
        }
    }

    pub fn with_component(mut self, def: ComponentDef) -> Self {
        self.components.insert(def.id.clone(), def);
        self
    }
}

impl ComponentHost for FakeComponentHost {
    fn codemod_root(&self) -> &Path {
        &self.codemod_root
    }

    fn components(&self) -> &BTreeMap<String, ComponentDef> {
        &self.components
    }

    fn component_stack(&self) -> &[String] {
        &self.component_stack
    }

    fn push_component(&mut self, id: &str) -> Result<(), String> {
        if self.component_stack.iter().any(|c| c == id) {
            return Err(format!("component cycle detected: {id}"));
        }
        self.component_stack.push(id.to_string());
        Ok(())
    }

    fn pop_component(&mut self, id: &str) {
        if self.component_stack.last().map(|s| s.as_str()) == Some(id) {
            self.component_stack.pop();
        }
    }

    fn graph_flat_json(&self) -> Option<&serde_json::Value> {
        self.graph_flat_json.as_ref()
    }

    fn graph_edges_json(&self) -> Option<&serde_json::Value> {
        self.graph_edges_json.as_ref()
    }

    fn set_graph_json(&mut self, flat: serde_json::Value, edges: serde_json::Value) {
        self.graph_flat_json = Some(flat);
        self.graph_edges_json = Some(edges);
    }

    fn flush_to_disk(&self) -> Result<Vec<String>, String> {
        if let Some(err) = &self.flush_error {
            return Err(err.clone());
        }
        Ok(self.flush_paths.clone())
    }

    fn finalize_tree(&mut self) -> Vec<FileChange> {
        std::mem::take(&mut self.pending_changes)
    }

    fn take_transcript(&mut self) -> Vec<TranscriptEvent> {
        std::mem::take(&mut self.transcript)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codemod_recipe_core::apply::file_change::{FileChange, IfExists};

    #[test]
    fn push_pop_detects_cycle() {
        let mut host = FakeComponentHost::new("/tmp/codemod");
        host.push_component("a").unwrap();
        host.push_component("b").unwrap();
        let err = host.push_component("a").unwrap_err();
        assert!(err.contains("cycle"), "{err}");
        assert_eq!(host.component_stack(), &["a".to_string(), "b".to_string()]);
        host.pop_component("b");
        assert_eq!(host.component_stack(), &["a".to_string()]);
        host.pop_component("a");
        assert!(host.component_stack().is_empty());
    }

    #[test]
    fn finalize_empty_and_nonempty() {
        let mut host = FakeComponentHost::new("/tmp/codemod");
        assert!(host.finalize_tree().is_empty());

        host.pending_changes.push(FileChange::Create {
            path: "x.rs".into(),
            content: "hi".into(),
            if_exists: IfExists::Fail,
            skipped: false,
        });
        host.transcript.push(TranscriptEvent::simple("create", "x.rs"));
        let changes = host.finalize_tree();
        assert_eq!(changes.len(), 1);
        assert!(host.pending_changes.is_empty());
        let t = host.take_transcript();
        assert_eq!(t.len(), 1);
        assert!(host.take_transcript().is_empty());
    }

    #[test]
    fn graph_roundtrip() {
        let mut host = FakeComponentHost::new("/tmp/codemod");
        host.set_graph_json(
            serde_json::json!([{"id": "a"}]),
            serde_json::json!([]),
        );
        assert_eq!(
            host.graph_flat_json(),
            Some(&serde_json::json!([{"id": "a"}]))
        );
        assert_eq!(host.graph_edges_json(), Some(&serde_json::json!([])));
    }

    #[test]
    fn minimal_finalize_path_without_registry() {
        // Criterion 2: facade finalize needs no RecipeRegistry / MCP / VS Code.
        let mut host = FakeComponentHost::new("/tmp/codemod");
        host.flush_paths = vec!["out.rs".into()];
        host.pending_changes.push(FileChange::Create {
            path: "out.rs".into(),
            content: "// generated".into(),
            if_exists: IfExists::Fail,
            skipped: false,
        });
        let applied = host.flush_to_disk().unwrap();
        assert_eq!(applied, vec!["out.rs".to_string()]);
        let changes = host.finalize_tree();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path(), "out.rs");
    }
}
