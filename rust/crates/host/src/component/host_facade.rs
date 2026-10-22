//! HostInner implements [`codemod_recipe_components::ComponentHost`] (D-025 F).

use std::collections::BTreeMap;
use std::path::Path;

use codemod_recipe_components::ComponentHost;
use codemod_recipe_core::apply::file_change::FileChange;
use codemod_recipe_lua::model::{ComponentDef, TranscriptEvent};

use crate::edit_session::working_tree::WorkingTree;
use crate::host_api::context::HostInner;

impl ComponentHost for HostInner {
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
        HostInner::flush_to_disk(self)
    }

    fn finalize_tree(&mut self) -> Vec<FileChange> {
        std::mem::replace(&mut self.tree, WorkingTree::new()).finalize()
    }

    fn take_transcript(&mut self) -> Vec<TranscriptEvent> {
        std::mem::take(&mut self.transcript)
    }
}
