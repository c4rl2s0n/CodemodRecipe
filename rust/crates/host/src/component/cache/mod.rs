//! Component scan cache for incremental re-identify.

mod body;
mod types;

pub use body::{locator_key, ComponentScanCache};
pub use types::{DetectResult, FileFingerprint, NewRoot, RootEntry, RootKey};
