use thiserror::Error;

use super::table_util::map_lua_error;
use crate::model::{
    ComponentDef, MapAsset, QueryLibAsset, Recipe, StackDef, TestSuiteDef, TypeDef, VariablesAsset,
};

#[derive(Debug, Error)]
pub enum LoadError {
    #[error("{0}")]
    Message(String),
    #[error("{message}")]
    Located {
        message: String,
        line: Option<u32>,
        column: Option<u32>,
    },
}

impl LoadError {
    pub fn location(&self) -> Option<(u32, u32)> {
        match self {
            Self::Located {
                line: Some(line),
                column: Some(column),
                ..
            } => Some((*line, *column)),
            _ => None,
        }
    }
}

impl From<mlua::Error> for LoadError {
    fn from(err: mlua::Error) -> Self {
        map_lua_error(err, "")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    Recipe,
    Map,
    Variables,
    QueryLib,
    Type,
    Stack,
    Component,
    TestSuite,
    /// Helper module (no top-level `id`); not catalogued, loadable via `require`.
    Library,
    Unknown,
}

#[derive(Debug)]
pub enum Asset {
    Recipe(Recipe),
    Map(MapAsset),
    Variables(VariablesAsset),
    QueryLib(QueryLibAsset),
    Type(TypeDef),
    Stack(StackDef),
    Component(ComponentDef),
    TestSuite(TestSuiteDef),
}

/// Outcome of loading a `.codemod` Luau module for the catalog.
#[derive(Debug)]
pub enum LoadResult {
    /// Boxed: `Asset` is ~260 bytes (Recipe carries maps/types/queries tables),
    /// and `LoadResult` is moved around per loaded file.
    Asset(Box<Asset>),
    /// Helper module — not registered as a catalog asset.
    Library,
}

