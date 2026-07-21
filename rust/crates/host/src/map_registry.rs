use crate::protocol::{DiagnosticSource, RecipeDiagnostic};
use serde_yaml::Value;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub struct MapLoadResult {
    pub maps_by_id: BTreeMap<String, BTreeMap<String, String>>,
    pub diagnostics: Vec<RecipeDiagnostic>,
}

pub fn load_maps(workspace_root: &Path, maps_directory: &Path) -> MapLoadResult {
    let mut maps_by_id: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut diagnostics = Vec::new();
    let mut id_sources: BTreeMap<String, Vec<DiagnosticSource>> = BTreeMap::new();

    if !maps_directory.is_dir() {
        return MapLoadResult {
            maps_by_id,
            diagnostics,
        };
    }

    let mut files = Vec::new();
    collect_yaml_files(maps_directory, &mut files);

    for path in files {
        let relative = relative_path(workspace_root, &path);
        match load_map_file(&path) {
            Ok((id, entries)) => {
                id_sources
                    .entry(id.clone())
                    .or_default()
                    .push(DiagnosticSource {
                        file: relative,
                        line: None,
                        column: None,
                    });
                maps_by_id.insert(id, entries);
            }
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    let rejected: Vec<String> = id_sources
        .iter()
        .filter(|(_, sources)| sources.len() > 1)
        .map(|(id, _)| id.clone())
        .collect();

    for id in &rejected {
        maps_by_id.remove(id);
        if let Some(sources) = id_sources.get(id) {
            diagnostics.push(RecipeDiagnostic::simple(
                "error",
                "E_DUPLICATE_MAP_ID",
                format!("Duplicate map id: {id}"),
                sources.clone(),
            ));
        }
    }

    MapLoadResult {
        maps_by_id,
        diagnostics,
    }
}

fn load_map_file(path: &Path) -> Result<(String, BTreeMap<String, String>), RecipeDiagnostic> {
    let relative = path_to_string(path);
    let text = std::fs::read_to_string(path).map_err(|e| RecipeDiagnostic::simple(
        "error",
        "E_MAP_PARSE",
        format!("Failed to read map file: {e}"),
        vec![DiagnosticSource {
            file: relative.clone(),
            line: None,
            column: None,
        }],
    ))?;

    let doc: Value = serde_yaml::from_str(&text).map_err(|e| RecipeDiagnostic::simple(
        "error",
        "E_MAP_PARSE",
        format!("Failed to parse map YAML: {e}"),
        vec![DiagnosticSource {
            file: relative.clone(),
            line: None,
            column: None,
        }],
    ))?;

    let Value::Mapping(root) = doc else {
        return Err(map_schema_error("Map file root must be a map", &relative));
    };

    let id = root
        .get("id")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| map_schema_error("Map file missing required \"id\"", &relative))?
        .to_string();

    let entries_value = root
        .get("entries")
        .ok_or_else(|| map_schema_error(&format!("Map \"{id}\" missing required \"entries\" map"), &relative))?;

    let Value::Mapping(entries_map) = entries_value else {
        return Err(map_schema_error(
            &format!("Map \"{id}\" missing required \"entries\" map"),
            &relative,
        ));
    };

    let mut entries = BTreeMap::new();
    for (key, value) in entries_map {
        let key = key.as_str().unwrap_or_default().to_string();
        let value = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            other => serde_yaml::to_string(other).unwrap_or_default(),
        };
        entries.insert(key, value);
    }

    Ok((id, entries))
}

fn map_schema_error(message: &str, file: &str) -> RecipeDiagnostic {
    RecipeDiagnostic::simple(
        "error",
        "E_MAP_SCHEMA",
        message.to_string(),
        vec![DiagnosticSource {
            file: file.to_string(),
            line: None,
            column: None,
        }],
    )
}

pub fn merge_maps(
    global: &BTreeMap<String, BTreeMap<String, String>>,
    inline: &BTreeMap<String, BTreeMap<String, String>>,
) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut merged: BTreeMap<String, BTreeMap<String, String>> = global
        .iter()
        .map(|(id, entries)| (id.clone(), entries.clone()))
        .collect();
    for (id, entries) in inline {
        merged
            .entry(id.clone())
            .or_default()
            .extend(entries.iter().map(|(k, v)| (k.clone(), v.clone())));
    }
    merged
}

pub fn warn_on_missing_map_ids(
    template: &str,
    file_path: &str,
    maps_by_id: &BTreeMap<String, BTreeMap<String, String>>,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    warn_legacy_map_references(template, file_path, maps_by_id, diagnostics);
    let converted = crate::template::convert_legacy_syntax(template);
    warn_jinja_map_references(&converted, file_path, maps_by_id, diagnostics);
}

fn warn_legacy_map_references(
    template: &str,
    file_path: &str,
    maps_by_id: &BTreeMap<String, BTreeMap<String, String>>,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    let mut index = 0;
    while let Some(start) = template[index..].find("{{$map") {
        let abs_start = index + start;
        let mut i = abs_start + "{{$map".len();
        while i < template.len() && template.as_bytes()[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= template.len() {
            break;
        }
        let quote = template.as_bytes()[i];
        if quote != b'\'' && quote != b'"' {
            index = i;
            continue;
        }
        i += 1;
        let id_start = i;
        while i < template.len() && template.as_bytes()[i] != quote {
            i += 1;
        }
        if i >= template.len() {
            break;
        }
        let map_id = &template[id_start..i];
        index = i + 1;

        if maps_by_id.contains_key(map_id) {
            continue;
        }
        push_map_id_warning(file_path, map_id, diagnostics);
    }
}

fn warn_jinja_map_references(
    template: &str,
    file_path: &str,
    maps_by_id: &BTreeMap<String, BTreeMap<String, String>>,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    let mut index = 0;
    while let Some(start) = template[index..].find("map(") {
        let abs_start = index + start;
        let rest = &template[abs_start + 4..];
        if let Some(map_id) = parse_quoted_map_id(rest) {
            if !maps_by_id.contains_key(&map_id) {
                push_map_id_warning(file_path, &map_id, diagnostics);
            }
            index = abs_start + 4 + map_id.len() + 2;
        } else {
            index = abs_start + 4;
        }
    }
}

fn parse_quoted_map_id(text: &str) -> Option<String> {
    let text = text.trim_start();
    let quote = text.as_bytes().first()?;
    if *quote != b'\'' && *quote != b'"' {
        return None;
    }
    let quote_char = *quote as char;
    let rest = &text[1..];
    let end = rest.find(quote_char)?;
    let map_id = rest[..end].to_string();
    if map_id.is_empty() {
        return None;
    }
    Some(map_id)
}

fn push_map_id_warning(
    file_path: &str,
    map_id: &str,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    if diagnostics.iter().any(|d| {
        d.code == "W_MAP_ID_NOT_FOUND" && d.message.contains(map_id) && d.sources.first().is_some_and(|s| s.file == file_path)
    }) {
        return;
    }
    diagnostics.push(RecipeDiagnostic::simple(
        "warning",
        "W_MAP_ID_NOT_FOUND",
        format!("Template references unknown map id: {map_id}"),
        vec![DiagnosticSource {
            file: file_path.to_string(),
            line: None,
            column: None,
        }],
    ));
}

fn collect_yaml_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_yaml_files(&path, out);
        } else if is_yaml(&path) {
            out.push(path);
        }
    }
}

fn is_yaml(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("yaml") | Some("yml")
    )
}

fn relative_path(workspace_root: &Path, absolute: &Path) -> String {
    let root = workspace_root.canonicalize().unwrap_or_else(|_| workspace_root.to_path_buf());
    let file = absolute.canonicalize().unwrap_or_else(|_| absolute.to_path_buf());
    if let Ok(rel) = file.strip_prefix(&root) {
        rel.to_string_lossy().to_string()
    } else {
        absolute.to_string_lossy().to_string()
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_workspace(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{name}_{}", std::process::id()))
    }

    #[test]
    fn loads_maps_by_id() {
        let workspace = temp_workspace("map_registry_ok");
        let maps_dir = workspace.join(".codemod/maps");
        std::fs::create_dir_all(&maps_dir).unwrap();
        std::fs::write(
            maps_dir.join("column_type.yaml"),
            r#"id: columnType
entries:
  int: intColumn
  String: textColumn
"#,
        )
        .unwrap();

        let result = load_maps(&workspace, &maps_dir);
        assert!(result
            .diagnostics
            .iter()
            .all(|d| d.severity != "error"));
        assert_eq!(
            result.maps_by_id["columnType"]["int"].as_str(),
            "intColumn"
        );

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn reports_duplicate_map_ids() {
        let workspace = temp_workspace("map_registry_dup");
        let maps_dir = workspace.join(".codemod/maps");
        std::fs::create_dir_all(&maps_dir).unwrap();
        std::fs::write(
            maps_dir.join("a.yaml"),
            "id: columnType\nentries:\n  int: intColumn\n",
        )
        .unwrap();
        std::fs::write(
            maps_dir.join("b.yaml"),
            "id: columnType\nentries:\n  String: textColumn\n",
        )
        .unwrap();

        let result = load_maps(&workspace, &maps_dir);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.code == "E_DUPLICATE_MAP_ID"));
        assert!(!result.maps_by_id.contains_key("columnType"));

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn merge_inline_maps_overlay_global() {
        let mut global = BTreeMap::new();
        let mut global_entries = BTreeMap::new();
        global_entries.insert("int".to_string(), "intColumn".to_string());
        global.insert("columnType".to_string(), global_entries);

        let mut inline = BTreeMap::new();
        let mut inline_entries = BTreeMap::new();
        inline_entries.insert("bool".to_string(), "boolColumn".to_string());
        inline.insert("columnType".to_string(), inline_entries);

        let merged = merge_maps(&global, &inline);
        assert_eq!(merged["columnType"]["int"], "intColumn");
        assert_eq!(merged["columnType"]["bool"], "boolColumn");
    }
}
