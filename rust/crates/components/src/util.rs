//! Small path helpers (kept local so this crate does not depend on host sandbox).

pub fn stem_from_path(path: &str) -> Option<String> {
    let path = path.trim().replace('\\', "/");
    let file = path.rsplit('/').next()?;
    let stem = file.rsplit_once('.').map(|(s, _)| s).unwrap_or(file);
    if stem.is_empty() {
        None
    } else {
        Some(stem.to_string())
    }
}
