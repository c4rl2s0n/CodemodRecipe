use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourcePatch {
    /// Byte offset into the original source.
    pub start: usize,
    /// Byte offset into the original source.
    pub end: usize,
    /// Replacement text inserted at [start..end].
    pub replacement: String,
    pub description: Option<String>,
}

impl SourcePatch {
    pub fn new(start: usize, end: usize, replacement: impl Into<String>) -> Self {
        Self {
            start,
            end,
            replacement: replacement.into(),
            description: None,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PatchError {
    #[error("patch range is invalid: start={start} end={end} source_len={source_len}")]
    InvalidRange {
        start: usize,
        end: usize,
        source_len: usize,
    },

    #[error("patches overlap: a=[{a_start},{a_end}) b=[{b_start},{b_end})")]
    Overlap {
        a_start: usize,
        a_end: usize,
        b_start: usize,
        b_end: usize,
    },
}

pub fn validate_non_overlapping(
    patches: &[SourcePatch],
    source_len: usize,
) -> Result<(), PatchError> {
    let mut ranges: Vec<(usize, usize)> = Vec::with_capacity(patches.len());
    for p in patches {
        if p.start > p.end || p.end > source_len {
            return Err(PatchError::InvalidRange {
                start: p.start,
                end: p.end,
                source_len,
            });
        }
        ranges.push((p.start, p.end));
    }

    ranges.sort_by_key(|(s, e)| (*s, *e));
    for w in ranges.windows(2) {
        let (a_start, a_end) = w[0];
        let (b_start, b_end) = w[1];
        if b_start < a_end {
            return Err(PatchError::Overlap {
                a_start,
                a_end,
                b_start,
                b_end,
            });
        }
    }
    Ok(())
}

/// Apply patches to the source. Patches are applied from end-to-start so offsets are stable.
pub fn apply_patches(source: &str, patches: &[SourcePatch]) -> Result<String, PatchError> {
    validate_non_overlapping(patches, source.len())?;

    let mut out = source.to_string();

    // End-to-start to keep byte offsets valid.
    let mut sorted: Vec<&SourcePatch> = patches.iter().collect();
    sorted.sort_by_key(|p| (p.start, p.end));
    for p in sorted.into_iter().rev() {
        out.replace_range(p.start..p.end, &p.replacement);
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn apply_patches_end_to_start() {
        let src = "abcdef";
        let patches = vec![
            SourcePatch::new(1, 3, "XX"), // a[1..3] -> XX
            SourcePatch::new(4, 4, "Z"),  // insert at 4
        ];
        let out = apply_patches(src, &patches).unwrap();
        assert_eq!(out, "aXXdZef");
    }

    #[test]
    fn detects_overlap() {
        let src = "abcdef";
        let patches = vec![SourcePatch::new(1, 4, "X"), SourcePatch::new(3, 5, "Y")];
        let err = apply_patches(src, &patches).unwrap_err();
        assert!(matches!(err, PatchError::Overlap { .. }));
    }

    #[test]
    fn preserves_declaration_order_for_same_offset_insertions() {
        let result = apply_patches(
            "ab",
            &[SourcePatch::new(1, 1, "X"), SourcePatch::new(1, 1, "Y")],
        )
        .unwrap();
        assert_eq!(result, "aXYb");
    }

    #[test]
    fn rejects_overlapping_replacement_patches() {
        let patches = vec![
            SourcePatch::new(1, 4, "first"),
            SourcePatch::new(2, 3, "second"),
        ];
        let err = validate_non_overlapping(&patches, 6).unwrap_err();
        assert!(matches!(err, PatchError::Overlap { .. }));
    }

    #[test]
    fn allows_adjacent_replacement_patches() {
        let patches = vec![SourcePatch::new(0, 1, "A"), SourcePatch::new(1, 2, "B")];
        validate_non_overlapping(&patches, 2).unwrap();
        assert_eq!(apply_patches("ab", &patches).unwrap(), "AB");
    }

    #[test]
    fn applies_interleaved_offsets_deterministically() {
        let result = apply_patches(
            "abcd",
            &[
                SourcePatch::new(3, 3, "Z"),
                SourcePatch::new(1, 1, "X"),
                SourcePatch::new(2, 2, "Y"),
            ],
        )
        .unwrap();
        assert_eq!(result, "aXbYcZd");
    }

    #[test]
    fn rejects_invalid_range() {
        let patches = vec![SourcePatch::new(3, 2, "X")];
        let err = apply_patches("ab", &patches).unwrap_err();
        assert!(matches!(err, PatchError::InvalidRange { .. }));
    }
}
