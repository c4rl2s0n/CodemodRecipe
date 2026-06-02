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
  static Future<Map<String, Object?>> changeToJson(
    FileChange change, {
    bool includeContents = true,
    bool includePatchReplacements = true,
    int snippetLines = 5,
  }) async {
    if (change is PatchFileChange) {
      final snippet = _snippetFromPatchChange(change, snippetLines);
      return {
        'path': change.path,
        'kind': 'edit',
        'isNew': false,
        'skipped': false,
        if (snippet.isNotEmpty) 'snippet': snippet,
        if (includeContents) 'original': change.source,
        if (includeContents) 'modified': change.generate(),
        'patches': [
          for (var i = 0; i < change.patches.length; i++)
            _patchToJson(
              i,
              change.patches[i],
              includeReplacement: includePatchReplacements,
            ),
        ],
      };
    }

    if (change is CreateFileChange) {
      String? original;
      if (includeContents) {
        final file = File(change.path);
        original = await file.exists() ? await file.readAsString() : '';
      }
      return {
        'path': change.path,
        'kind': 'create',
        'isNew': !change.exists,
        'skipped': !change.hasChanges,
        'snippet': _snippetFromText(change.content, snippetLines),
        if (includeContents) 'original': original,
        if (includeContents) 'modified': change.content,
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
    List<FileChange> changes, {
    bool includeContents = true,
    bool includePatchReplacements = true,
    int snippetLines = 5,
  }) async {
    return [
      for (final change in changes)
        await changeToJson(
          change,
          includeContents: includeContents,
          includePatchReplacements: includePatchReplacements,
          snippetLines: snippetLines,
        ),
    ];
  }

  static Map<String, Object?> _patchToJson(
    int index,
    SourcePatch patch, {
    required bool includeReplacement,
  }) {
    final replacement = patch.replacement;
    return {
      'index': index,
      'offset': patch.offset,
      'length': patch.length,
      if (includeReplacement) 'replacement': replacement,
      if (!includeReplacement)
        'replacementPreview': replacement.length > 200
            ? '${replacement.substring(0, 200)}…'
            : replacement,
      'description': patch.description,
    };
  }

  static String _snippetFromPatchChange(PatchFileChange change, int maxLines) {
    if (change.patches.isEmpty) return '';
    final replacement = change.patches.first.replacement;
    return _snippetFromText(replacement, maxLines);
  }

  static String _snippetFromText(String source, int maxLines) {
    if (source.isEmpty) return '';
    final normalized = source.replaceAll('\r\n', '\n');
    final lines = normalized.split('\n');
    final take = lines.take(maxLines).join('\n').trimRight();
    return take;
  }
}
