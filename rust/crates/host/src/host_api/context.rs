use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;

use codemod_recipe_engine::{LanguageRegistry, RegistryConfig};
use codemod_recipe_lua::model::{Recipe, TranscriptEvent};

use crate::engine::working_tree::WorkingTree;
use crate::sandbox::path_sandbox::PathSandbox;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunKind {
    Preview,
    Apply,
}

pub struct RecipeRunResult {
    pub changes: Vec<codemod_recipe_core::apply::file_change::FileChange>,
    pub transcript: Vec<TranscriptEvent>,
    pub saw_post_exec: bool,
}

pub struct RecipeRunRequest {
    pub kind: RunKind,
    pub args: BTreeMap<String, serde_json::Value>,
    pub maps: BTreeMap<String, BTreeMap<String, String>>,
    pub vars: BTreeMap<String, BTreeMap<String, String>>,
    pub queries: BTreeMap<String, String>,
    pub recipes: BTreeMap<String, Recipe>,
    pub workspace_root: PathBuf,
    pub language_config: RegistryConfig,
    pub codemod_root: PathBuf,
}

pub(crate) const MAX_OPS: usize = 50_000;

pub(crate) type SharedHost = Rc<Mutex<HostInner>>;

pub(crate) struct HostInner {
    pub(crate) kind: RunKind,
    pub(crate) tree: WorkingTree,
    pub(crate) sandbox: PathSandbox,
    pub(crate) language_registry: LanguageRegistry,
    pub(crate) args: BTreeMap<String, serde_json::Value>,
    pub(crate) maps: BTreeMap<String, BTreeMap<String, String>>,
    pub(crate) vars: BTreeMap<String, BTreeMap<String, String>>,
    pub(crate) queries: BTreeMap<String, String>,
    pub(crate) recipes: BTreeMap<String, Recipe>,
    pub(crate) recipe_path: Option<PathBuf>,
    pub(crate) codemod_root: PathBuf,
    pub(crate) transcript: Vec<TranscriptEvent>,
    pub(crate) saw_post_exec: bool,
    pub(crate) op_count: usize,
}

impl HostInner {
    pub(crate) fn current_content(&mut self, path: &str) -> Result<String, String> {
        self.tree.current_content(&self.sandbox, path)
    }

    pub(crate) fn apply_edit<F>(&mut self, path: &str, transform: F) -> Result<(), String>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        self.tree.apply_edit(&self.sandbox, path, transform)
    }

    pub(crate) fn create(
        &mut self,
        path: &str,
        content: String,
        if_exists: codemod_recipe_core::apply::file_change::IfExists,
    ) -> Result<(), String> {
        self.tree.create(&self.sandbox, path, content, if_exists)
    }

    pub(crate) fn delete(
        &mut self,
        path: &str,
        if_missing: codemod_recipe_core::apply::file_change::IfMissing,
    ) -> Result<(), String> {
        self.tree.delete(&self.sandbox, path, if_missing)
    }

    pub(crate) fn flush_to_disk(&self) -> Result<Vec<String>, String> {
        self.tree.flush_to_disk(&self.sandbox)
    }

    pub(crate) fn reload_from_disk(&mut self, path: &str) -> Result<(), String> {
        self.tree.reload_from_disk(&self.sandbox, path)
    }
}

pub(crate) fn bump_ops(host: &mut HostInner) -> Result<(), String> {
    host.op_count += 1;
    if host.op_count > MAX_OPS {
        Err("recipe exceeded operation budget".to_string())
    } else {
        Ok(())
    }
}
