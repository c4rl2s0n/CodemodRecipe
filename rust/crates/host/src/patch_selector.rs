use codemod_recipe_core::patch::SourcePatch;
use codemod_recipe_core::patch::{apply_patches, PatchError};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default)]
pub struct FileSelection {
    pub include: bool,
    pub patch_indices: Option<Vec<usize>>,
}

pub fn parse_selection(value: &serde_json::Value) -> BTreeMap<String, FileSelection> {
    let mut result = BTreeMap::new();
    let Some(files) = value.get("files").and_then(|v| v.as_object()) else {
        return result;
    };
    for (path, entry) in files {
        let include = entry
            .get("include")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        let patch_indices = entry.get("patches").and_then(|v| {
            v.as_array().map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_u64().map(|n| n as usize))
                    .collect()
            })
        });
        result.insert(
            path.clone(),
            FileSelection {
                include,
                patch_indices,
            },
        );
    }
    result
}

pub fn select_patches(
    source: &str,
    patches: &[SourcePatch],
    indices: &[usize],
) -> Result<(String, Vec<SourcePatch>), PatchError> {
    let wanted: std::collections::BTreeSet<usize> = indices.iter().copied().collect();
    let selected: Vec<SourcePatch> = patches
        .iter()
        .enumerate()
        .filter(|(i, _)| wanted.contains(i))
        .map(|(_, p)| p.clone())
        .collect();
    let modified = apply_patches(source, &selected)?;
    Ok((modified, selected))
}

pub fn apply_selection(
    path: &str,
    source: &str,
    patches: &[SourcePatch],
    selection: &BTreeMap<String, FileSelection>,
) -> Result<Option<(String, Vec<SourcePatch>)>, PatchError> {
    let file_selection = selection.get(path);
    if matches!(file_selection, Some(s) if !s.include) {
        return Ok(None);
    }
    if patches.is_empty() {
        return Ok(None);
    }
    if let Some(indices) = file_selection.and_then(|s| s.patch_indices.as_ref()) {
        let (modified, selected) = select_patches(source, patches, indices)?;
        if selected.is_empty() {
            return Ok(None);
        }
        return Ok(Some((modified, selected)));
    }
    Ok(Some((apply_patches(source, patches)?, patches.to_vec())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use codemod_recipe_core::patch::SourcePatch;

    #[test]
    fn keeps_only_requested_patch_indices() {
        let source = "abcdef";
        let patches = vec![SourcePatch::new(1, 1, "X"), SourcePatch::new(3, 3, "Y")];
        let (modified, selected) = select_patches(source, &patches, &[1]).unwrap();
        assert_eq!(modified, "abcYdef");
        assert_eq!(selected.len(), 1);
    }

    #[test]
    fn drops_file_when_include_is_false() {
        let patches = vec![SourcePatch::new(0, 1, "A")];
        let mut selection = BTreeMap::new();
        selection.insert(
            "a.dart".to_string(),
            FileSelection {
                include: false,
                patch_indices: None,
            },
        );
        let result = apply_selection("a.dart", "ab", &patches, &selection).unwrap();
        assert!(result.is_none());
    }
}
