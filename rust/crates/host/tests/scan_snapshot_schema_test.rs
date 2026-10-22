//! D-014 / D-025 E: committed dogfood snapshot must keep the documented key shape.
//! Order is checked on the raw file (serde_json::Value sorts object keys).

use serde_json::Value;
use std::path::PathBuf;

fn repo_snapshot() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../.codemod/.cache/scan-snapshot.json")
}

fn first_top_level_key_offset(raw: &str, key: &str) -> Option<usize> {
    // Pretty-printed snapshot: keys appear as `  "key":` at the start of a line.
    let needle = format!("\n  \"{key}\":");
    raw.find(&needle).or_else(|| {
        let alt = format!("{{\n  \"{key}\":");
        raw.find(&alt)
    })
}

#[test]
fn committed_scan_snapshot_has_required_top_level_keys() {
    let path = repo_snapshot();
    if !path.is_file() {
        // Fresh checkouts may omit the cache until the next component_scan.
        return;
    }
    let raw = std::fs::read_to_string(&path).expect("read scan-snapshot.json");
    let value: Value = serde_json::from_str(&raw).expect("parse scan-snapshot.json");
    let obj = value.as_object().expect("snapshot object");
    for key in ["meta", "coverage", "relations", "instances", "edges", "findings"] {
        assert!(
            obj.contains_key(key),
            "scan-snapshot.json missing top-level key {key:?}"
        );
    }
    // Documented order (D-020): meta → coverage → relations → rest
    let meta = first_top_level_key_offset(&raw, "meta").expect("meta key in file");
    let coverage = first_top_level_key_offset(&raw, "coverage").expect("coverage key in file");
    let relations = first_top_level_key_offset(&raw, "relations").expect("relations key in file");
    assert!(
        meta < coverage && coverage < relations,
        "top-level key order should be meta → coverage → relations (offsets {meta}/{coverage}/{relations})"
    );
}
