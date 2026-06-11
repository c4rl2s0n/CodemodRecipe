import '../operation.dart';

/// Filters planned changes down to a user-selected subset.
///
/// The VS Code extension lets users review a preview and choose which
/// individual patches (for edited files) or which whole files (for created
/// files) to keep before applying. [PatchSelector] turns that selection into a
/// reduced list of [FileChange]s that can be applied with normal recipe logic.
class PatchSelector {
  const PatchSelector._();

  /// Returns a [PatchFileChange] containing only the patches at [indices].
  ///
  /// Indices refer to positions in [change.patches]. Out-of-range or duplicate
  /// indices are ignored. Order of the original patch list is preserved.
  static PatchFileChange selectPatches(
    PatchFileChange change,
    Iterable<int> indices,
  ) {
    final wanted = indices.toSet();
    final selected = [
      for (var i = 0; i < change.patches.length; i++)
        if (wanted.contains(i)) change.patches[i],
    ];
    return PatchFileChange(
      path: change.path,
      source: change.source,
      patches: selected,
    );
  }

  /// Applies a [selection] map to a list of planned [changes].
  ///
  /// The [selection] is keyed by file path. Each entry may contain:
  /// - `include`: whether the file should be applied at all (defaults to true).
  /// - `patches`: a list of patch indices to keep for [PatchFileChange]s. When
  ///   omitted, all patches for that file are kept.
  ///
  /// Paths absent from [selection] are kept unchanged (include all). Changes
  /// whose `include` is false, or whose selected patch list is empty, are
  /// dropped from the result.
  static List<FileChange> apply(
    List<FileChange> changes,
    Map<String, FileSelection> selection,
  ) {
    final result = <FileChange>[];

    for (final change in changes) {
      final fileSelection = selection[change.path];

      if (fileSelection != null && !fileSelection.include) {
        continue;
      }

      if (change is PatchFileChange) {
        final indices = fileSelection?.patchIndices;
        if (indices == null) {
          result.add(change);
          continue;
        }
        final filtered = selectPatches(change, indices);
        if (filtered.hasChanges) {
          result.add(filtered);
        }
        continue;
      }

      result.add(change);
    }

    return result;
  }
}

/// User selection for a single file in a preview.
class FileSelection {
  /// Whether to apply this file at all.
  final bool include;

  /// Patch indices to keep for edited files, or null to keep all patches.
  final List<int>? patchIndices;

  /// Creates a file selection.
  const FileSelection({this.include = true, this.patchIndices});

  /// Builds a [FileSelection] from a JSON map.
  factory FileSelection.fromJson(Map<String, Object?> json) {
    final rawPatches = json['patches'];
    return FileSelection(
      include: json['include'] as bool? ?? true,
      patchIndices: rawPatches is List
          ? [for (final value in rawPatches) (value as num).toInt()]
          : null,
    );
  }
}
