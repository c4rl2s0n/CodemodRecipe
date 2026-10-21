//! HostCommand request enum (protocol wire).

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(tag = "command")]
pub enum HostCommand {
    #[serde(rename = "list")]
    List,
    #[serde(rename = "reload")]
    Reload,
    #[serde(rename = "validate")]
    Validate {
        #[serde(default)]
        recipe: Option<String>,
    },
    #[serde(rename = "describe")]
    Describe { recipe: String },
    #[serde(rename = "preview")]
    Preview {
        #[serde(default)]
        recipe: Option<String>,
        #[serde(default, rename = "inlineRecipe")]
        inline_recipe: Option<serde_json::Value>,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
        #[serde(default, rename = "snippetLines")]
        snippet_lines: Option<u32>,
    },
    #[serde(rename = "apply")]
    Apply {
        #[serde(default)]
        recipe: Option<String>,
        #[serde(default, rename = "inlineRecipe")]
        inline_recipe: Option<serde_json::Value>,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
        #[serde(rename = "previewToken")]
        preview_token: String,
        #[serde(default)]
        selection: serde_json::Value,
    },
    #[serde(rename = "diff")]
    Diff {
        #[serde(default)]
        recipe: Option<String>,
        #[serde(default, rename = "inlineRecipe")]
        inline_recipe: Option<serde_json::Value>,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
        path: String,
    },
    #[serde(rename = "bootstrap")]
    Bootstrap {
        #[serde(default)]
        force: bool,
        #[serde(default, rename = "editPolicy")]
        edit_policy: Option<String>,
        #[serde(default)]
        companions: Vec<String>,
    },
    #[serde(rename = "deriveArgs")]
    DeriveArgs {
        recipe: String,
        source: String,
        #[serde(default)]
        language: Option<String>,
        #[serde(default)]
        path: Option<String>,
        #[serde(default, rename = "cursorOffset")]
        cursor_offset: u64,
        #[serde(default, rename = "selectionStart")]
        selection_start: u64,
        #[serde(default, rename = "selectionEnd")]
        selection_end: u64,
        #[serde(default)]
        context: std::collections::BTreeMap<String, String>,
    },
    #[serde(rename = "filterExplorerRecipes")]
    FilterExplorerRecipes { path: String, kind: String },
    #[serde(rename = "dumpAst")]
    DumpAst {
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        language: Option<String>,
        #[serde(default = "default_true", rename = "namedOnly")]
        named_only: bool,
    },
    #[serde(rename = "debugQuery")]
    DebugQuery {
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        language: Option<String>,
        query: String,
        #[serde(default = "default_true")]
        instrument: bool,
        #[serde(default, rename = "includeSexp")]
        include_sexp: bool,
    },
    #[serde(rename = "generateQuery")]
    GenerateQuery {
        #[serde(default)]
        path: Option<String>,
        #[serde(default)]
        source: Option<String>,
        #[serde(default)]
        language: Option<String>,
        start: u64,
        #[serde(default)]
        end: u64,
        #[serde(default, rename = "includeTextPredicates")]
        include_text_predicates: bool,
        #[serde(default, rename = "captureLeaf")]
        capture_leaf: Option<String>,
        #[serde(default, rename = "maxDepth")]
        max_depth: Option<u64>,
    },
    #[serde(rename = "resolveStaticPath")]
    ResolveStaticPath { template: String },
    /// Resolve `@capture` names from an inline query or registered query id.
    #[serde(rename = "queryCaptures")]
    QueryCaptures {
        #[serde(default)]
        query: Option<String>,
        #[serde(default, rename = "queryId")]
        query_id: Option<String>,
    },
    #[serde(rename = "componentScan")]
    ComponentScan,
    #[serde(rename = "componentIncrementalScan")]
    ComponentIncrementalScan {
        #[serde(default, rename = "dirtyPaths")]
        dirty_paths: Vec<String>,
        #[serde(default, rename = "detectChanges")]
        detect_changes: bool,
        #[serde(default)]
        force: bool,
    },
    #[serde(rename = "componentListInstances")]
    ComponentListInstances,
    #[serde(rename = "componentCoverage")]
    ComponentCoverage,
    /// Render the relations graph as mermaid text (`level`: `kinds` default, or
    /// `instances`). Scan-on-demand when no scan cache exists.
    #[serde(rename = "componentGraph")]
    ComponentGraph {
        #[serde(default)]
        level: Option<String>,
        /// Optional workspace-relative path to write the mermaid text.
        #[serde(default)]
        out: Option<String>,
    },
    #[serde(rename = "componentInvokeOperation")]
    ComponentInvokeOperation {
        #[serde(rename = "instanceId")]
        instance_id: String,
        operation: String,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
        #[serde(default)]
        preview: bool,
    },
    #[serde(rename = "componentScaffold")]
    ComponentScaffold {
        #[serde(rename = "componentId")]
        component_id: String,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
        #[serde(default)]
        preview: bool,
    },
    /// Dry-run or apply a single component hook (`identify` / `parse` / `verify` /
    /// `toSummary` / `scaffold` / `operation`). MCP exposes separate tools that set
    /// `preview` true vs false for allowlisting.
    #[serde(rename = "componentInvokeHook")]
    ComponentInvokeHook {
        #[serde(rename = "componentId")]
        component_id: String,
        hook: String,
        #[serde(default = "default_true")]
        preview: bool,
        #[serde(default)]
        locator: Option<serde_json::Value>,
        #[serde(default)]
        instance: Option<serde_json::Value>,
        #[serde(default)]
        operation: Option<String>,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
        /// Required when `preview` is false (apply). From a prior dry-run.
        #[serde(default, rename = "previewToken")]
        preview_token: Option<String>,
    },
    #[serde(rename = "describeComponent")]
    DescribeComponent {
        #[serde(rename = "componentId")]
        component_id: String,
    },
    /// Dry-run or apply a stack-level operation (`operations.<name>` on `stack.luau`).
    /// Instance-less; MCP exposes dry-run vs apply as separate tools for allowlisting.
    #[serde(rename = "stackInvokeOperation")]
    StackInvokeOperation {
        operation: String,
        #[serde(default = "default_true")]
        preview: bool,
        #[serde(default)]
        args: std::collections::BTreeMap<String, serde_json::Value>,
        /// Required when `preview` is false (apply). From a prior dry-run.
        #[serde(default, rename = "previewToken")]
        preview_token: Option<String>,
    },
    /// Run `*.test.luau` suites (fixture temp workspaces; session assets load
    /// from the process codemod root, but fixture scans do not write `.cache/`).
    /// `filter` matches suite id, suite `target`, `suite/test`, or a bare test
    /// name substring.
    #[serde(rename = "runTests")]
    RunTests {
        #[serde(default)]
        filter: Option<String>,
    },

    #[serde(rename = "componentSnapshot")]
    ComponentSnapshot {
        /// Codemod-root-relative path. Empty / omitted → `.cache/scan-snapshot.json`.
        #[serde(default)]
        path: Option<String>,
    },
}

fn default_true() -> bool {
    true
}
