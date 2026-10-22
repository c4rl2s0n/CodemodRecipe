//! Narrow views over [`HostInner`] (D-025 F / HostInner facet sketch).
//!
//! Locking stays on [`SharedHost`]; these types only scope field access so
//! component / recipe / query code paths do not reach across the whole god-object
//! at call sites. Not separate mutexes — extraction still deferred.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use codemod_recipe_core::apply::file_change::{IfExists, IfMissing};
use codemod_recipe_engine::LanguageRegistry;
use codemod_recipe_lua::model::TranscriptEvent;

use super::context::{bump_ops, HostInner, RunKind, SharedHost};
use crate::edit_session::working_tree::WorkingTree;
use crate::query::ast_cache::CachedTree;
use crate::sandbox::path_sandbox::PathSandbox;

/// Recipe preview/apply / host_api handlers facet.
pub(crate) struct RecipeSession<'a> {
    inner: &'a mut HostInner,
}

impl<'a> RecipeSession<'a> {
    pub(crate) fn from_inner(inner: &'a mut HostInner) -> Self {
        Self { inner }
    }

    pub(crate) fn with_host<R>(
        host: &SharedHost,
        f: impl FnOnce(RecipeSession<'_>) -> Result<R, String>,
    ) -> Result<R, String> {
        let mut inner = host.lock().map_err(|e| e.to_string())?;
        f(RecipeSession::from_inner(&mut inner))
    }

    /// Escape hatch for helpers that still take `&mut HostInner` (e.g. engine ops).
    pub(crate) fn host_mut(&mut self) -> &mut HostInner {
        self.inner
    }

    pub(crate) fn kind(&self) -> RunKind {
        self.inner.kind
    }

    pub(crate) fn tree(&mut self) -> &mut WorkingTree {
        &mut self.inner.tree
    }

    pub(crate) fn sandbox(&self) -> &PathSandbox {
        &self.inner.sandbox
    }

    pub(crate) fn args(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.inner.args
    }

    pub(crate) fn maps(&self) -> &BTreeMap<String, BTreeMap<String, String>> {
        &self.inner.maps
    }

    pub(crate) fn vars(&self) -> &BTreeMap<String, BTreeMap<String, String>> {
        &self.inner.vars
    }

    pub(crate) fn queries(&self) -> &BTreeMap<String, String> {
        &self.inner.queries
    }

    pub(crate) fn transcript(&mut self) -> &mut Vec<TranscriptEvent> {
        &mut self.inner.transcript
    }

    pub(crate) fn recipe_path(&self) -> Option<&PathBuf> {
        self.inner.recipe_path.as_ref()
    }

    pub(crate) fn codemod_root(&self) -> &Path {
        &self.inner.codemod_root
    }

    pub(crate) fn bump_ops(&mut self) -> Result<(), String> {
        bump_ops(self.inner)
    }

    pub(crate) fn set_saw_post_exec(&mut self, value: bool) {
        self.inner.saw_post_exec = value;
    }

    pub(crate) fn current_content(&mut self, path: &str) -> Result<String, String> {
        self.inner.current_content(path)
    }

    pub(crate) fn apply_edit<F>(&mut self, path: &str, transform: F) -> Result<(), String>
    where
        F: FnOnce(&str) -> Result<String, String>,
    {
        self.inner.apply_edit(path, transform)
    }

    pub(crate) fn create(
        &mut self,
        path: &str,
        content: String,
        if_exists: IfExists,
    ) -> Result<(), String> {
        self.inner.create(path, content, if_exists)
    }

    pub(crate) fn delete(&mut self, path: &str, if_missing: IfMissing) -> Result<(), String> {
        self.inner.delete(path, if_missing)
    }

    pub(crate) fn flush_to_disk(&self) -> Result<Vec<String>, String> {
        self.inner.flush_to_disk()
    }

    pub(crate) fn reload_from_disk(&mut self, path: &str) -> Result<(), String> {
        self.inner.reload_from_disk(path)
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

    pub(crate) fn with_host<R>(
        host: &SharedHost,
        f: impl FnOnce(QueryIntel<'_>) -> Result<R, String>,
    ) -> Result<R, String> {
        let mut inner = host.lock().map_err(|e| e.to_string())?;
        f(QueryIntel::from_inner(&mut inner))
    }

    pub(crate) fn language_registry(&self) -> &LanguageRegistry {
        &self.inner.language_registry
    }

    pub(crate) fn language_registry_mut(&mut self) -> &mut LanguageRegistry {
        &mut self.inner.language_registry
    }

    pub(crate) fn queries(&self) -> &BTreeMap<String, String> {
        &self.inner.queries
    }

    pub(crate) fn recipe_path(&self) -> Option<&PathBuf> {
        self.inner.recipe_path.as_ref()
    }

    pub(crate) fn codemod_root(&self) -> &Path {
        &self.inner.codemod_root
    }

    pub(crate) fn current_content(&mut self, path: &str) -> Result<String, String> {
        self.inner.current_content(path)
    }

    pub(crate) fn transcript(&mut self) -> &mut Vec<TranscriptEvent> {
        &mut self.inner.transcript
    }

    pub(crate) fn cached_tree(
        &mut self,
        path: &str,
        source: &str,
        language: Option<&str>,
    ) -> Result<CachedTree, String> {
        let lang_id = self
            .inner
            .language_registry
            .resolve_language_id(language, path)
            .map_err(|e| e.to_string())?;
        let lang = {
            let engine = self
                .inner
                .language_registry
                .get(&lang_id)
                .map_err(|e| e.to_string())?;
            engine.language()
        };
        self.inner.ast_cache.get_or_parse(path, source, &lang)
    }
}
