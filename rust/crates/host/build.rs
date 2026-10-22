use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BootstrapToml {
    source: Vec<Source>,
}

#[derive(Debug, Deserialize)]
struct Source {
    root: String,
    dest: String,
    /// Exact file or directory basenames to skip (e.g. "docs", "book").
    #[serde(default)]
    exclude: Vec<String>,
    /// File extensions to skip without the dot (e.g. "js").
    #[serde(default)]
    exclude_extensions: Vec<String>,
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir
        .join("../../..")
        .canonicalize()
        .unwrap_or_else(|_| manifest_dir.join("../../.."));
    let bootstrap_toml = repo_root.join("export/bootstrap.toml");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));

    println!("cargo:rerun-if-changed={}", bootstrap_toml.display());

    let raw = fs::read_to_string(&bootstrap_toml).unwrap_or_else(|e| {
        panic!("failed to read {}: {e}", bootstrap_toml.display());
    });
    let config: BootstrapToml = toml::from_str(&raw).unwrap_or_else(|e| {
        panic!("failed to parse {}: {e}", bootstrap_toml.display());
    });

    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    for source in &config.source {
        let root = repo_root.join(&source.root);
        println!("cargo:rerun-if-changed={}", root.display());
        if !root.is_dir() {
            panic!(
                "bootstrap source root missing: {} (from {})",
                root.display(),
                source.root
            );
        }
        walk_dir(
            &root,
            &root,
            &mut entries,
            &source.dest,
            &source.exclude,
            &source.exclude_extensions,
        );
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    // Detect accidental duplicate embed destinations.
    for w in entries.windows(2) {
        if w[0].0 == w[1].0 {
            panic!(
                "duplicate bootstrap embed path {} (from {} and {})",
                w[0].0,
                w[0].1.display(),
                w[1].1.display()
            );
        }
    }

    let mut manifest = String::from(
        "#[derive(Debug, Clone, Copy)]\n\
         pub struct ExportedFile {\n\
             pub path: &'static str,\n\
             pub content: &'static str,\n\
         }\n\n\
         pub static EXPORTED_FILES: &[ExportedFile] = &[\n",
    );

    for (i, (rel_path, src_path)) in entries.iter().enumerate() {
        let embed_name = format!("embed_{i}.txt");
        let embed_path = out_dir.join(&embed_name);
        // Follow symlinks when copying content (docs symlink → docs/usage).
        let bytes = fs::read(src_path).unwrap_or_else(|e| {
            panic!("failed to read {}: {e}", src_path.display());
        });
        fs::write(&embed_path, &bytes).unwrap_or_else(|e| {
            panic!("failed to write embed {}: {e}", embed_path.display());
        });
        manifest.push_str(&format!(
            "    ExportedFile {{ path: \"{rel_path}\", content: include_str!(\"{embed_name}\") }},\n",
            rel_path = rel_path.replace('\\', "/"),
        ));
    }

    manifest.push_str("];\n");

    fs::write(out_dir.join("bootstrap_manifest.rs"), manifest)
        .expect("write bootstrap_manifest.rs");

    for (_, src_path) in &entries {
        println!("cargo:rerun-if-changed={}", src_path.display());
    }
}

fn walk_dir(
    root: &Path,
    dir: &Path,
    out: &mut Vec<(String, PathBuf)>,
    dest_prefix: &str,
    exclude: &[String],
    exclude_extensions: &[String],
) {
    let read_dir = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in read_dir {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if exclude.iter().any(|e| e == &name_str) {
            continue;
        }
        // Do not follow directory symlinks into excluded trees when walking skills.
        let meta = fs::symlink_metadata(&path).unwrap_or_else(|e| {
            panic!("metadata {}: {e}", path.display());
        });
        if meta.file_type().is_symlink() && path.is_dir() {
            // Skip symlink dirs (e.g. docs → usage); content comes from another source.
            continue;
        }
        if path.is_dir() {
            walk_dir(root, &path, out, dest_prefix, exclude, exclude_extensions);
        } else if path.is_file() || (meta.file_type().is_symlink() && path.exists()) {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if exclude_extensions.iter().any(|e| e == ext) {
                    continue;
                }
            }
            let rel = path
                .strip_prefix(root)
                .expect("path under root")
                .to_string_lossy()
                .replace('\\', "/");
            let embed_path = if dest_prefix.is_empty() {
                rel
            } else if rel.is_empty() {
                dest_prefix.to_string()
            } else {
                format!("{dest_prefix}/{rel}")
            };
            out.push((embed_path, path));
        }
    }
}
