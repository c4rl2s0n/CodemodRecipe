//! Narrow views over [`HostInner`] (D-025 F / HostInner facet sketch).
//!
//! Locking stays on [`SharedHost`]; these types only scope field access so
//! component / recipe / query code paths do not reach across the whole god-object
//! at call sites. Not separate mutexes — extraction still deferred.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use codemod_recipe_engine::LanguageRegistry;
use codemod_recipe_lua::model::{ComponentDef, Recipe, TranscriptEvent};

use super::context::{HostInner, RunKind, SharedHost};
use crate::edit_session::working_tree::WorkingTree;
use crate::query::ast_cache::AstParseCache;
use crate::sandbox::path_sandbox::PathSandbox;

/// Component scan / hooks / ops / delegate facet.
pub(crate) struct ComponentSession<'a> {
    inner: &'a mut HostInner,
}

impl<'a> ComponentSession<'a> {
    pub(crate) fn from_inner(inner: &'a mut HostInner) -> Self {
        Self { inner }
    }

    pub(crate) fn with_host<R>(
        host: &SharedHost,
        f: impl FnOnce(ComponentSession<'_>) -> Result<R, String>,
    ) -> Result<R, String> {
        let mut inner = host.lock().map_err(|e| e.to_string())?;
        f(ComponentSession::from_inner(&mut inner))
    }

    pub(crate) fn components(&self) -> &BTreeMap<String, ComponentDef> {
        &self.inner.components
    }

    pub(crate) fn codemod_root(&self) -> &Path {
        &self.inner.codemod_root
    }

    pub(crate) fn component_stack(&self) -> &[String] {
        &self.inner.component_stack
    }

    pub(crate) fn push_component(&mut self, id: &str) -> Result<(), String> {
        if self.inner.component_stack.iter().any(|c| c == id) {
            return Err(format!("component cycle detected: {id}"));
        }
        self.inner.component_stack.push(id.to_string());
        Ok(())
    }

    pub(crate) fn pop_component(&mut self, id: &str) {
        if self.inner.component_stack.last().map(|s| s.as_str()) == Some(id) {
            self.inner.component_stack.pop();
        }
    }

    pub(crate) fn graph_flat_json(&self) -> Option<&serde_json::Value> {
        self.inner.graph_flat_json.as_ref()
    }

    pub(crate) fn graph_edges_json(&self) -> Option<&serde_json::Value> {
        self.inner.graph_edges_json.as_ref()
    }

    pub(crate) fn set_graph_json(
        &mut self,
        flat: serde_json::Value,
        edges: serde_json::Value,
    ) {
        self.inner.graph_flat_json = Some(flat);
        self.inner.graph_edges_json = Some(edges);
    }
}

/// Recipe preview/apply / host_api handlers facet.
pub(crate) struct RecipeSession<'a> {
    inner: &'a mut HostInner,
}

impl<'a> RecipeSession<'a> {
    pub(crate) fn from_inner(inner: &'a mut HostInner) -> Self {
        Self { inner }
    }

    #[allow(dead_code)] // Phase 1: boundary ready; handlers migrate later
    pub(crate) fn with_host<R>(
        host: &SharedHost,
        f: impl FnOnce(RecipeSession<'_>) -> Result<R, String>,
    ) -> Result<R, String> {
        let mut inner = host.lock().map_err(|e| e.to_string())?;
        f(RecipeSession::from_inner(&mut inner))
    }

    #[allow(dead_code)]
    pub(crate) fn kind(&self) -> RunKind {
        self.inner.kind
    }

    #[allow(dead_code)]
    pub(crate) fn tree(&mut self) -> &mut WorkingTree {
        &mut self.inner.tree
    }

    #[allow(dead_code)]
    pub(crate) fn sandbox(&self) -> &PathSandbox {
        &self.inner.sandbox
    }

    #[allow(dead_code)]
    pub(crate) fn recipes(&self) -> &BTreeMap<String, Recipe> {
        &self.inner.recipes
    }

    #[allow(dead_code)]
    pub(crate) fn transcript(&mut self) -> &mut Vec<TranscriptEvent> {
        &mut self.inner.transcript
    }

    #[allow(dead_code)]
    pub(crate) fn recipe_path(&self) -> Option<&PathBuf> {
        self.inner.recipe_path.as_ref()
    }
}

/// Query / AST cache facet.
pub(crate) struct QueryIntel<'a> {
    inner: &'a mut HostInner,
}

impl<'a> QueryIntel<'a> {
    pub(crate) fn from_inner(inner: &'a mut HostInner) -> Self {
        Self { inner }
    }

    #[allow(dead_code)]
    pub(crate) fn with_host<R>(
        host: &SharedHost,
        f: impl FnOnce(QueryIntel<'_>) -> Result<R, String>,
    ) -> Result<R, String> {
        let mut inner = host.lock().map_err(|e| e.to_string())?;
        f(QueryIntel::from_inner(&mut inner))
    }

    #[allow(dead_code)]
    pub(crate) fn language_registry(&self) -> &LanguageRegistry {
        &self.inner.language_registry
    }

    #[allow(dead_code)]
    pub(crate) fn ast_cache(&mut self) -> &mut AstParseCache {
        &mut self.inner.ast_cache
    }
}
