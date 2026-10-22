//! Component scan: identify/parse, graph phases, incremental.

mod full;
mod graph_phases;
mod identify;
mod incremental;
mod summary;
mod types;

pub use full::{component_scan, component_scan_inner};
pub use incremental::component_incremental_scan;
pub use types::{ComponentIncrementalScanRequest, ScanData};
