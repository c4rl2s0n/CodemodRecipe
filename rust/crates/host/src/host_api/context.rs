use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Mutex;

use codemod_recipe_engine::{LanguageRegistry, RegistryConfig};
use codemod_recipe_lua::model::{ComponentDef, Recipe, TranscriptEvent};

use crate::edit_session::working_tree::WorkingTree;
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

/// Failed `apply()` with any transcript events recorded before the error.
#[derive(Debug, Clone)]
pub struct RecipeApplyError {
    pub message: String,
    pub transcript: Vec<TranscriptEvent>,
}

impl std::fmt::Display for RecipeApplyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for RecipeApplyError {}

pub struct RecipeRunRequest {
    pub kind: RunKind,
    pub args: BTreeMap<String, serde_json::Value>,
    pub maps: BTreeMap<String, BTreeMap<String, String>>,
    pub vars: BTreeMap<String, BTreeMap<String, String>>,
    pub queries: BTreeMap<String, String>,
    pub recipes: BTreeMap<String, Recipe>,
    pub components: BTreeMap<String, ComponentDef>,
    pub workspace_root: PathBuf,
    pub codemod_root: PathBuf,
    pub language_config: RegistryConfig,
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
    pub(crate) components: BTreeMap<String, ComponentDef>,
    pub(crate) component_stack: Vec<String>,
    pub(crate) recipe_path: Option<PathBuf>,
    pub(crate) codemod_root: PathBuf,
    pub(crate) transcript: Vec<TranscriptEvent>,
    pub(crate) saw_post_exec: bool,
    pub(crate) op_count: usize,
    pub(crate) ast_cache: crate::query::ast_cache::AstParseCache,
    /// Pre-operation graph snapshot (flat instance JSON array + edges JSON array).
    /// Set by stack/component operation/scaffold invoke so `c:component():operation`
    /// fan-out can rebuild the graph facade in the shared sandbox.
    pub(crate) graph_flat_json: Option<serde_json::Value>,
    pub(crate) graph_edges_json: Option<serde_json::Value>,
}

impl HostInner {
    pub(crate) fn current_content(&mut self, path: &str) -> Result<String, String> {
        self.tree.current_content(&self.sandbox, path)
    }

    pub(crate) fn apply_edit<F>(&mut self, path: &str, transform: F) -> Result<(), String>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        self.ast_cache.invalidate(path);
        self.tree.apply_edit(&self.sandbox, path, transform)
    }

    pub(crate) fn create(
        &mut self,
        path: &str,
        content: String,
        if_exists: codemod_recipe_core::apply::file_change::IfExists,
    ) -> Result<(), String> {
        self.ast_cache.invalidate(path);
        self.tree.create(&self.sandbox, path, content, if_exists)
    }

    pub(crate) fn delete(
        &mut self,
        path: &str,
        if_missing: codemod_recipe_core::apply::file_change::IfMissing,
    ) -> Result<(), String> {
        self.ast_cache.invalidate(path);
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
