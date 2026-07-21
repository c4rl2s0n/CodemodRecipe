use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let export_root = manifest_dir.join("../../..").join("export");
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));

    println!("cargo:rerun-if-changed={}", export_root.display());

    let mut entries: Vec<(String, PathBuf)> = Vec::new();
    if export_root.is_dir() {
        walk_dir(&export_root, &export_root, &mut entries);
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

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
        fs::copy(src_path, &embed_path).unwrap_or_else(|e| {
            panic!("failed to copy export file {}: {e}", src_path.display());
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

fn walk_dir(export_root: &Path, dir: &Path, out: &mut Vec<(String, PathBuf)>) {
    let read_dir = fs::read_dir(dir).unwrap_or_else(|e| panic!("read {}: {e}", dir.display()));
    for entry in read_dir {
        let entry = entry.expect("dir entry");
        let path = entry.path();
        if path.is_dir() {
            walk_dir(export_root, &path, out);
        } else if path.is_file() {
            let rel = path
                .strip_prefix(export_root)
                .expect("path under export root");
            out.push((rel.to_string_lossy().into_owned(), path));
        }
    }
}
