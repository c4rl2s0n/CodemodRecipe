use crate::protocol::{DiagnosticSource, RecipeDiagnostic};
use serde_yaml::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

pub struct AssetLoadResult {
    pub maps_by_id: BTreeMap<String, BTreeMap<String, String>>,
    pub vars_by_id: BTreeMap<String, BTreeMap<String, String>>,
    /// Absolute paths of YAML files classified as recipes.
    pub recipe_paths: Vec<PathBuf>,
    pub diagnostics: Vec<RecipeDiagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AssetKind {
    Recipe,
    Map,
    Variables,
}

/// Recursively scan `codemod_root` for YAML and classify by schema (not by directory).
pub fn load_codemod_assets(workspace_root: &Path, codemod_root: &Path) -> AssetLoadResult {
    let mut maps_by_id: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut vars_by_id: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut recipe_paths = Vec::new();
    let mut diagnostics = Vec::new();
    let mut map_id_sources: BTreeMap<String, Vec<DiagnosticSource>> = BTreeMap::new();
    let mut var_id_sources: BTreeMap<String, Vec<DiagnosticSource>> = BTreeMap::new();

    if !codemod_root.is_dir() {
        return AssetLoadResult {
            maps_by_id,
            vars_by_id,
            recipe_paths,
            diagnostics,
        };
    }

    let mut files = Vec::new();
    collect_yaml_files(codemod_root, &mut files);

    for path in files {
        let relative = relative_path(workspace_root, &path);
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                diagnostics.push(RecipeDiagnostic::simple(
                    "error",
                    "E_ASSET_PARSE",
                    format!("Failed to read YAML file: {e}"),
                    vec![DiagnosticSource {
                        file: relative,
                        line: None,
                        column: None,
                    }],
                ));
                continue;
            }
        };

        // Detect duplicate keys in map:/values: blocks from source text before YAML
        // parsers collapse them.
        let mut duplicate_keys = false;
        for field in ["map", "values"] {
            if let Some(dup) = find_duplicate_keys_in_block(&text, field) {
                diagnostics.push(schema_error(
                    &format!("Duplicate key \"{dup}\" in \"{field}\""),
                    &relative,
                ));
                duplicate_keys = true;
            }
        }
        if duplicate_keys {
            continue;
        }

        let doc: Value = match serde_yaml::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                diagnostics.push(RecipeDiagnostic::simple(
                    "error",
                    "E_ASSET_PARSE",
                    format!("Failed to parse YAML: {e}"),
                    vec![DiagnosticSource {
                        file: relative,
                        line: None,
                        column: None,
                    }],
                ));
                continue;
            }
        };

        let Value::Mapping(root) = &doc else {
            continue;
        };

        match classify_root(root, &relative) {
            Ok(None) => continue,
            Ok(Some(AssetKind::Recipe)) => {
                recipe_paths.push(path);
            }
            Ok(Some(AssetKind::Map)) => match parse_keyed_string_map(&text, root, "map", &relative)
            {
                Ok((id, entries)) => {
                    map_id_sources
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
            },
            Ok(Some(AssetKind::Variables)) => {
                match parse_keyed_string_map(&text, root, "values", &relative) {
                    Ok((id, entries)) => {
                        var_id_sources
                            .entry(id.clone())
                            .or_default()
                            .push(DiagnosticSource {
                                file: relative,
                                line: None,
                                column: None,
                            });
                        vars_by_id.insert(id, entries);
                    }
                    Err(diagnostic) => diagnostics.push(diagnostic),
                }
            }
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }

    reject_duplicate_ids(
        &mut maps_by_id,
        &map_id_sources,
        "E_DUPLICATE_MAP_ID",
        "Duplicate map id",
        &mut diagnostics,
    );
    reject_duplicate_ids(
        &mut vars_by_id,
        &var_id_sources,
        "E_DUPLICATE_VAR_ID",
        "Duplicate variables id",
        &mut diagnostics,
    );

    AssetLoadResult {
        maps_by_id,
        vars_by_id,
        recipe_paths,
        diagnostics,
    }
}

fn classify_root(
    root: &serde_yaml::Mapping,
    relative: &str,
) -> Result<Option<AssetKind>, RecipeDiagnostic> {
    let has_steps = root.contains_key("steps");
    let has_map = root.contains_key("map");
    let has_values = root.contains_key("values");
    let source = vec![DiagnosticSource {
        file: relative.to_string(),
        line: None,
        column: None,
    }];

    if has_steps && (has_map || has_values) {
        return Err(RecipeDiagnostic::simple(
            "error",
            "E_AMBIGUOUS_ASSET",
            "YAML asset cannot combine steps with map or values".to_string(),
            source,
        ));
    }
    if has_map && has_values {
        return Err(RecipeDiagnostic::simple(
            "error",
            "E_AMBIGUOUS_ASSET",
            "YAML asset cannot define both map and values".to_string(),
            source,
        ));
    }
    if has_steps {
        return Ok(Some(AssetKind::Recipe));
    }
    if has_map {
        return Ok(Some(AssetKind::Map));
    }
    if has_values {
        return Ok(Some(AssetKind::Variables));
    }
    Ok(None)
}

fn parse_keyed_string_map(
    text: &str,
    root: &serde_yaml::Mapping,
    field: &str,
    relative: &str,
) -> Result<(String, BTreeMap<String, String>), RecipeDiagnostic> {
    let id = root
        .get("id")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(|| {
            schema_error(
                &format!("Asset with \"{field}\" missing required \"id\""),
                relative,
            )
        })?
        .to_string();

    if let Some(dup) = find_duplicate_keys_in_block(text, field) {
        return Err(schema_error(
            &format!("Duplicate key \"{dup}\" in \"{field}\" of \"{id}\""),
            relative,
        ));
    }

    let payload = root.get(field).ok_or_else(|| {
        schema_error(
            &format!("Asset \"{id}\" missing required \"{field}\" map"),
            relative,
        )
    })?;

    let Value::Mapping(entries_map) = payload else {
        return Err(schema_error(
            &format!("Asset \"{id}\" field \"{field}\" must be a map"),
            relative,
        ));
    };

    let mut entries = BTreeMap::new();
    for (key, value) in entries_map {
        let key = key.as_str().unwrap_or_default().to_string();
        if key.is_empty() {
            continue;
        }
        let value = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => String::new(),
            other => serde_yaml::to_string(other)
                .unwrap_or_default()
                .trim()
                .to_string(),
        };
        entries.insert(key, value);
    }

    Ok((id, entries))
}

/// Detect duplicate sibling keys under a top-level `map:` / `values:` block via indentation.
fn find_duplicate_keys_in_block(text: &str, field: &str) -> Option<String> {
    let header = format!("{field}:");
    let mut block_indent: Option<usize> = None;
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut in_block = false;
    let mut header_indent = 0usize;

    for line in text.lines() {
        if !in_block {
            let trimmed = line.trim_start();
            if trimmed == header || trimmed.starts_with(&header) {
                in_block = true;
                header_indent = leading_spaces(line);
            }
            continue;
        }

        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        let indent = leading_spaces(line);
        if indent <= header_indent {
            break;
        }
        let key_indent = *block_indent.get_or_insert(indent);
        if indent != key_indent {
            if indent < key_indent {
                break;
            }
            continue;
        }
        let Some(key) = key_from_yaml_line(line.trim_start()) else {
            continue;
        };
        if !seen.insert(key.clone()) {
            return Some(key);
        }
    }
    None
}

fn key_from_yaml_line(trimmed: &str) -> Option<String> {
    if trimmed.starts_with('-') {
        return None;
    }
    let without_comment = trimmed.split('#').next()?.trim();
    let (key_part, _) = without_comment.split_once(':')?;
    let key = key_part.trim().trim_matches('"').trim_matches('\'').to_string();
    if key.is_empty() {
        return None;
    }
    Some(key)
}

fn leading_spaces(line: &str) -> usize {
    line.chars().take_while(|c| *c == ' ').count()
}

fn reject_duplicate_ids(
    by_id: &mut BTreeMap<String, BTreeMap<String, String>>,
    id_sources: &BTreeMap<String, Vec<DiagnosticSource>>,
    code: &'static str,
    message_prefix: &str,
    diagnostics: &mut Vec<RecipeDiagnostic>,
) {
    let rejected: Vec<String> = id_sources
        .iter()
        .filter(|(_, sources)| sources.len() > 1)
        .map(|(id, _)| id.clone())
        .collect();

    for id in &rejected {
        by_id.remove(id);
        if let Some(sources) = id_sources.get(id) {
            diagnostics.push(RecipeDiagnostic::simple(
                "error",
                code,
                format!("{message_prefix}: {id}"),
                sources.clone(),
            ));
        }
    }
}

fn schema_error(message: &str, file: &str) -> RecipeDiagnostic {
    RecipeDiagnostic::simple(
        "error",
        "E_ASSET_SCHEMA",
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
        d.code == "W_MAP_ID_NOT_FOUND"
            && d.message.contains(map_id)
            && d.sources
                .first()
                .is_some_and(|s| s.file == file_path)
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
    let root = workspace_root
        .canonicalize()
        .unwrap_or_else(|_| workspace_root.to_path_buf());
    let file = absolute
        .canonicalize()
        .unwrap_or_else(|_| absolute.to_path_buf());
    if let Ok(rel) = file.strip_prefix(&root) {
        rel.to_string_lossy().to_string()
    } else {
        absolute.to_string_lossy().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_workspace(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("{name}_{}", std::process::id()))
    }

    #[test]
    fn loads_maps_by_id_with_map_key() {
        let workspace = temp_workspace("map_registry_ok");
        let maps_dir = workspace.join(".codemod/maps");
        std::fs::create_dir_all(&maps_dir).unwrap();
        std::fs::write(
            maps_dir.join("column_type.yaml"),
            r#"id: columnType
map:
  int: intColumn
  String: textColumn
"#,
        )
        .unwrap();

        let result = load_codemod_assets(&workspace, &workspace.join(".codemod"));
        assert!(result.diagnostics.iter().all(|d| d.severity != "error"));
        assert_eq!(
            result.maps_by_id["columnType"]["int"].as_str(),
            "intColumn"
        );

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn loads_variables_and_maps_from_any_directory() {
        let workspace = temp_workspace("asset_any_dir");
        let root = workspace.join(".codemod");
        std::fs::create_dir_all(root.join("custom")).unwrap();
        std::fs::write(
            root.join("custom/paths.yaml"),
            r#"id: paths
values:
  feature_root: lib/features
"#,
        )
        .unwrap();
        std::fs::write(
            root.join("custom/types.yaml"),
            r#"id: paths
map:
  x: int
"#,
        )
        .unwrap();

        let result = load_codemod_assets(&workspace, &root);
        assert!(result.diagnostics.iter().all(|d| d.severity != "error"));
        assert_eq!(result.vars_by_id["paths"]["feature_root"], "lib/features");
        assert_eq!(result.maps_by_id["paths"]["x"], "int");

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn entries_key_is_not_loaded_as_map() {
        let workspace = temp_workspace("map_entries_ignored");
        let maps_dir = workspace.join(".codemod/maps");
        std::fs::create_dir_all(&maps_dir).unwrap();
        std::fs::write(
            maps_dir.join("legacy.yaml"),
            r#"id: columnType
entries:
  int: intColumn
"#,
        )
        .unwrap();

        let result = load_codemod_assets(&workspace, &workspace.join(".codemod"));
        assert!(!result.maps_by_id.contains_key("columnType"));

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn reports_duplicate_map_ids() {
        let workspace = temp_workspace("map_registry_dup");
        let maps_dir = workspace.join(".codemod/maps");
        std::fs::create_dir_all(&maps_dir).unwrap();
        std::fs::write(
            maps_dir.join("a.yaml"),
            "id: columnType\nmap:\n  int: intColumn\n",
        )
        .unwrap();
        std::fs::write(
            maps_dir.join("b.yaml"),
            "id: columnType\nmap:\n  String: textColumn\n",
        )
        .unwrap();

        let result = load_codemod_assets(&workspace, &workspace.join(".codemod"));
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.code == "E_DUPLICATE_MAP_ID"));
        assert!(!result.maps_by_id.contains_key("columnType"));

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn reports_duplicate_var_ids() {
        let workspace = temp_workspace("var_registry_dup");
        let _ = std::fs::remove_dir_all(&workspace);
        let vars_dir = workspace.join(".codemod/variables");
        std::fs::create_dir_all(&vars_dir).unwrap();
        std::fs::write(
            vars_dir.join("a.yaml"),
            "id: paths\nvalues:\n  feature_root: lib/a\n",
        )
        .unwrap();
        std::fs::write(
            vars_dir.join("b.yaml"),
            "id: paths\nvalues:\n  feature_root: lib/b\n",
        )
        .unwrap();

        let result = load_codemod_assets(&workspace, &workspace.join(".codemod"));
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == "E_DUPLICATE_VAR_ID"),
            "diagnostics: {:?}",
            result.diagnostics
        );
        assert!(!result.vars_by_id.contains_key("paths"));

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn allows_same_id_across_map_and_variables() {
        let workspace = temp_workspace("cross_type_same_id");
        let _ = std::fs::remove_dir_all(&workspace);
        let root = workspace.join(".codemod");
        std::fs::create_dir_all(root.join("anywhere")).unwrap();
        std::fs::write(
            root.join("anywhere/shared_map.yaml"),
            "id: shared\nmap:\n  key: mapValue\n",
        )
        .unwrap();
        std::fs::write(
            root.join("anywhere/shared_var.yaml"),
            "id: shared\nvalues:\n  key: varValue\n",
        )
        .unwrap();

        let result = load_codemod_assets(&workspace, &root);
        assert!(
            result.diagnostics.iter().all(|d| d.severity != "error"),
            "diagnostics: {:?}",
            result.diagnostics
        );
        assert!(result
            .diagnostics
            .iter()
            .all(|d| d.code != "E_DUPLICATE_MAP_ID" && d.code != "E_DUPLICATE_VAR_ID"));
        assert_eq!(result.maps_by_id["shared"]["key"], "mapValue");
        assert_eq!(result.vars_by_id["shared"]["key"], "varValue");

        let _ = std::fs::remove_dir_all(workspace);
    }

    #[test]
    fn reports_duplicate_keys_within_values_block() {
        let workspace = temp_workspace("var_dup_keys");
        let _ = std::fs::remove_dir_all(&workspace);
        let dir = workspace.join(".codemod/variables");
        std::fs::create_dir_all(&dir).unwrap();
        let contents = "id: paths\nvalues:\n  feature_root: a\n  feature_root: b\n";
        std::fs::write(dir.join("paths.yaml"), contents).unwrap();
        assert_eq!(
            find_duplicate_keys_in_block(contents, "values").as_deref(),
            Some("feature_root")
        );

        let result = load_codemod_assets(&workspace, &workspace.join(".codemod"));
        assert!(
            result
                .diagnostics
                .iter()
                .any(|d| d.code == "E_ASSET_SCHEMA" && d.message.contains("Duplicate key")),
            "diagnostics: {:?}",
            result.diagnostics
        );
        assert!(!result.vars_by_id.contains_key("paths"));

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

    #[test]
    fn detects_duplicate_keys_helper() {
        let text = "id: paths\nvalues:\n  a: 1\n  a: 2\n";
        assert_eq!(find_duplicate_keys_in_block(text, "values").as_deref(), Some("a"));
    }
}
