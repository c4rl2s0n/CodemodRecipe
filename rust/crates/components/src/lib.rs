//! Component domain crate (D-025 F): `ComponentHost` facade, relations, types.
//!
//! Luau session / `RecipeRegistry` scan orchestration remains in `codemod_recipe_host`
//! (entry adapter) and calls into this crate through [`ComponentHost`].

pub mod util;
pub mod graph_render;
pub mod host_facade;
pub mod membership;
pub mod relation_metrics;
pub mod relations;
pub mod types;

pub use host_facade::{ComponentHost, FakeComponentHost};
pub use membership::{member_kinds, validate_membership_forest, MembershipForest};
pub use relations::{ComponentEdge, KindEdge};
pub use types::*;
