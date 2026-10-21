//! Filesystem walk helpers for loading Luau assets under `.codemod/`.

use std::path::{Path, PathBuf};

pub(crate) fn collect_lua_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    visit(root, &mut out);
    out.sort();
    // `.codemod/tests/fixtures/` holds workspace fixtures for test suites —
    // never load them as assets (a fixture may itself contain a .codemod tree).
    let fixtures_root = root.join("tests").join("fixtures");
    out.retain(|p| !p.starts_with(&fixtures_root));
    out
}

fn visit(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            visit(&path, out);
        } else if matches!(
            path.extension().and_then(|s| s.to_str()),
            Some("luau") | Some("lua")
        ) && !is_declaration_file(&path)
        {
            out.push(path);
        }
    }
}

fn is_declaration_file(path: &Path) -> bool {
    codemod_recipe_lua::is_declaration_file_name(path)
}

pub(crate) fn relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
