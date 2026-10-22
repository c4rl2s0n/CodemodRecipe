//! Sole `SharedHost` / `HostInner` touchpoint inside `component/` (D-025 F).
//!
//! Luau `c:*` still needs [`SharedHost`]; project to [`ComponentHost`] as soon as
//! the component call stack needs stack / graph / finalize.

use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Mutex;

use codemod_recipe_engine::{LanguageRegistry, RegistryConfig};
use codemod_recipe_lua::model::ComponentDef;

use super::types::ComponentInstance;
use crate::component::ComponentHost;
use crate::edit_session::working_tree::WorkingTree;
use crate::host_api::context::{HostInner, RunKind, SharedHost};
use crate::sandbox::path_sandbox::PathSandbox;

/// Inputs needed to construct a Luau host session for component ops.
pub(crate) struct HostBuildParams {
    pub kind: RunKind,
    pub args: BTreeMap<String, serde_json::Value>,
    pub maps: BTreeMap<String, BTreeMap<String, String>>,
    pub vars: BTreeMap<String, BTreeMap<String, String>>,
    pub queries: BTreeMap<String, String>,
    pub recipes: BTreeMap<String, codemod_recipe_lua::model::Recipe>,
    pub components: BTreeMap<String, ComponentDef>,
    pub workspace_root: std::path::PathBuf,
    pub codemod_root: std::path::PathBuf,
    pub language_config: RegistryConfig,
}

pub(crate) fn make_shared_host(params: HostBuildParams) -> Result<SharedHost, String> {
    Ok(Rc::new(Mutex::new(HostInner {
        kind: params.kind,
        tree: WorkingTree::new(),
        sandbox: PathSandbox::new(params.workspace_root),
        language_registry: LanguageRegistry::with_config(params.language_config),
        args: params.args,
        maps: params.maps,
        vars: params.vars,
        queries: params.queries,
        recipes: params.recipes,
        components: params.components,
        component_stack: Vec::new(),
        recipe_path: None,
        codemod_root: params.codemod_root,
        transcript: Vec::new(),
        saw_post_exec: false,
        op_count: 0,
        ast_cache: crate::query::ast_cache::AstParseCache::default(),
        graph_flat_json: None,
        graph_edges_json: None,
    })))
}

/// Lock the shared host and run `f` against the [`ComponentHost`] facade.
pub(crate) fn with_component_host<R>(
    host: &SharedHost,
    f: impl FnOnce(&mut dyn ComponentHost) -> Result<R, String>,
) -> Result<R, String> {
    let mut inner = host.lock().map_err(|e| e.to_string())?;
    f(&mut *inner)
}

pub(crate) fn set_graph_snapshot(
    host: &SharedHost,
    instances: &[ComponentInstance],
    edges: &[crate::component::relations::ComponentEdge],
) -> Result<(), String> {
    let flat = crate::component::relations::flatten_instances(instances);
    let flat_json: Vec<serde_json::Value> = flat.iter().map(|f| f.to_json()).collect();
    let edges_json = serde_json::to_value(edges).map_err(|e| e.to_string())?;
    with_component_host(host, |h| {
        h.set_graph_json(serde_json::Value::Array(flat_json), edges_json);
        Ok(())
    })
}
