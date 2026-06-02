import 'dart:io';

import '../operation.dart';
import '../patch_helpers.dart';

/// Builds structured, JSON-serializable previews of planned file changes.
///
/// The VS Code extension consumes this data to drive its native diff viewer
/// (original vs. modified text) and to render per-patch checkboxes for
/// selective application.
class DiffService {
  const DiffService._();

  /// Serializes a single planned [change] into a JSON-friendly map.
  ///
  /// The shape is:
  /// ```json
  /// {
  ///   "path": "lib/foo.dart",
  ///   "kind": "edit" | "create",
  ///   "isNew": false,
  ///   "skipped": false,
  ///   "original": "...source...",
  ///   "modified": "...source...",
  ///   "patches": [
  ///     {"index": 0, "offset": 12, "length": 0,
  ///      "replacement": "void foo() {}", "description": "Add method foo"}
  ///   ]
  /// }
  /// ```
  ///
  /// For created files, `patches` is empty and the diff is a whole-file
  /// comparison between any existing content and the rendered content.
  static Future<Map<String, Object?>> changeToJson(FileChange change) async {
    if (change is PatchFileChange) {
      return {
        'path': change.path,
        'kind': 'edit',
        'isNew': false,
        'skipped': false,
        'original': change.source,
        'modified': change.generate(),
        'patches': [
          for (var i = 0; i < change.patches.length; i++)
            _patchToJson(i, change.patches[i]),
        ],
      };
    }

    if (change is CreateFileChange) {
      final file = File(change.path);
      final original = await file.exists() ? await file.readAsString() : '';
      return {
        'path': change.path,
        'kind': 'create',
        'isNew': !change.exists,
        'skipped': !change.hasChanges,
        'original': original,
        'modified': change.content,
        'patches': const <Map<String, Object?>>[],
      };
    }

    // Fallback for custom FileChange implementations: expose the preview text.
    return {
      'path': change.path,
      'kind': 'other',
      'isNew': false,
      'skipped': !change.hasChanges,
      'original': '',
      'modified': '',
      'preview': change.preview(),
      'patches': const <Map<String, Object?>>[],
    };
  }

  /// Serializes a list of [changes] into JSON-friendly maps.
  static Future<List<Map<String, Object?>>> changesToJson(
    List<FileChange> changes,
  ) async {
    return [for (final change in changes) await changeToJson(change)];
  }

  static Map<String, Object?> _patchToJson(int index, SourcePatch patch) {
    return {
      'index': index,
      'offset': patch.offset,
      'length': patch.length,
      'replacement': patch.replacement,
      'description': patch.description,
    };
  }
}
