/// Source patch generation and application utilities.
///
/// Provides a lightweight patch system for making edits to source code.
// ignore_for_file: dangling_library_doc_comments

import 'dart:io';

import 'package:analyzer/dart/ast/ast.dart';

/// Represents a single source edit against an original source string.
///
/// Patches are applied in reverse order (from end of file to start)
/// to preserve offset positions.
class SourcePatch {
  /// The byte offset in the original source where the edit begins.
  final int offset;

  /// The number of bytes to delete (0 for pure insertion).
  final int length;

  /// The replacement text to insert.
  final String replacement;

  /// Optional description of what this patch does.
  final String? description;

  /// Creates a patch at [offset] that replaces [length] bytes.
  const SourcePatch(
    this.offset,
    this.length,
    this.replacement, {
    this.description,
  });

  @override
  String toString() {
    if (description != null) {
      return 'SourcePatch($offset, $length, "$replacement") // $description';
    }
    return 'SourcePatch($offset, $length, "$replacement")';
  }
}

/// Applies [patches] to [source] and returns the generated source.
///
/// Patches are sorted from the end of the source toward the beginning so their
/// original offsets remain valid while applying changes.
String applyPatches(String source, List<SourcePatch> patches) {
  final sortedPatches = _stablePatchOrder(patches, descending: true);

  var result = source;
  for (final patch in sortedPatches) {
    result =
        result.substring(0, patch.offset) +
        patch.replacement +
        result.substring(patch.offset + patch.length);
  }

  return result;
}

/// Throws if any non-empty patch ranges overlap.
///
/// Equal-offset insertions are allowed because they do not replace source text.
void validateNonOverlappingPatches(List<SourcePatch> patches) {
  final sortedPatches = List<SourcePatch>.from(patches)
    ..sort((a, b) => a.offset.compareTo(b.offset));

  SourcePatch? previous;
  for (final patch in sortedPatches) {
    if (previous == null) {
      previous = patch;
      continue;
    }

    final previousEnd = previous.offset + previous.length;
    final overlaps =
        previousEnd > previous.offset && patch.offset < previousEnd;

    if (overlaps) {
      throw StateError(
        'Overlapping patches at offsets ${previous.offset} and ${patch.offset}',
      );
    }

    previous = patch;
  }
}

List<SourcePatch> _stablePatchOrder(
  List<SourcePatch> patches, {
  required bool descending,
}) {
  final indexedPatches = patches.indexed.toList()
    ..sort((a, b) {
      final offsetCompare = descending
          ? b.$2.offset.compareTo(a.$2.offset)
          : a.$2.offset.compareTo(b.$2.offset);
      if (offsetCompare != 0) return offsetCompare;

      // Equal-offset insertions apply in reverse input order so the final
      // generated source preserves transform declaration order.
      return descending ? b.$1.compareTo(a.$1) : a.$1.compareTo(b.$1);
    });

  return indexedPatches.map((entry) => entry.$2).toList();
}

/// Creates an insertion patch immediately after [node].
SourcePatch insertAfterNode(AstNode node, String code, {String? description}) {
  return SourcePatch(
    node.end,
    0,
    code,
    description: description ?? 'Insert after ${node.runtimeType}',
  );
}

/// Creates an insertion patch immediately before [node].
SourcePatch insertBeforeNode(AstNode node, String code, {String? description}) {
  return SourcePatch(
    node.offset,
    0,
    code,
    description: description ?? 'Insert before ${node.runtimeType}',
  );
}

/// Creates a patch that replaces [node] with [code].
SourcePatch replaceNode(AstNode node, String code, {String? description}) {
  return SourcePatch(
    node.offset,
    node.length,
    code,
    description: description ?? 'Replace ${node.runtimeType}',
  );
}

/// Creates a patch that deletes [node].
SourcePatch deleteNode(AstNode node, {String? description}) {
  return SourcePatch(
    node.offset,
    node.length,
    '',
    description: description ?? 'Delete ${node.runtimeType}',
  );
}

/// Creates an insertion patch before a class closing brace offset.
SourcePatch insertIntoClass(
  int classEndOffset,
  String code, {
  String? description,
}) {
  return SourcePatch(
    classEndOffset - 1,
    0,
    code,
    description: description ?? 'Insert into class',
  );
}

/// Formats a compact human-readable preview for [patches].
String previewPatches(String source, List<SourcePatch> patches) {
  final buffer = StringBuffer();
  final sortedPatches = _stablePatchOrder(patches, descending: false);

  for (final patch in sortedPatches) {
    final contextStart = patch.offset > 20 ? patch.offset - 20 : 0;
    final contextEnd = patch.offset + patch.length + 20 < source.length
        ? patch.offset + patch.length + 20
        : source.length;

    final before = source.substring(contextStart, patch.offset);
    final after = source.substring(patch.offset + patch.length, contextEnd);

    buffer.writeln('---');
    if (patch.description != null) {
      buffer.writeln('# ${patch.description}');
    }
    buffer.writeln('At offset ${patch.offset}:${patch.length}');
    buffer.writeln('```');
    buffer.write(before);
    buffer.write('[${patch.replacement}]');
    buffer.write(after);
    buffer.writeln('\n```');
    buffer.writeln();
  }

  return buffer.toString();
}

/// Applies [patches] to [filePath] and writes the result back to disk.
///
/// Returns false when the file cannot be read or written.
Future<bool> applyPatchesToFile(
  String filePath,
  List<SourcePatch> patches,
) async {
  try {
    final file = File(filePath);
    if (!await file.exists()) {
      stderr.writeln('Error: File not found: $filePath');
      return false;
    }

    final source = await file.readAsString();
    final result = applyPatches(source, patches);
    await file.writeAsString(result);
    return true;
  } catch (e) {
    stderr.writeln('Error applying patches to $filePath: $e');
    return false;
  }
}

/// Generates a simple unified-diff-style preview for [patches].
String generateDiff(
  String originalPath,
  String original,
  List<SourcePatch> patches,
) {
  final buffer = StringBuffer();
  buffer.writeln('--- $originalPath');
  buffer.writeln('+++ $originalPath');
  buffer.writeln();

  final sortedPatches = _stablePatchOrder(patches, descending: false);

  for (final patch in sortedPatches) {
    final contextLines = 3;
    final lines = original.split('\n');

    var offsetCounter = 0;
    var contextStartLine = 0;
    for (var i = 0; i < lines.length; i++) {
      if (offsetCounter + lines[i].length + 1 > patch.offset) {
        contextStartLine = i - contextLines;
        if (contextStartLine < 0) contextStartLine = 0;
        break;
      }
      offsetCounter += lines[i].length + 1;
    }

    final hunkStart = contextStartLine + 1;
    final hunkLength = contextLines * 2 + 1;
    buffer.writeln('@@ -$hunkStart,$hunkLength +$hunkStart,$hunkLength @@');

    for (
      var i = contextStartLine;
      i < contextStartLine + hunkLength && i < lines.length;
      i++
    ) {
      if (i >= 0) {
        final isDeletedLine =
            offsetCounter >= patch.offset &&
            offsetCounter < patch.offset + patch.length;

        if (isDeletedLine) {
          buffer.writeln('-${lines[i]}');
        } else if (i == contextStartLine + contextLines) {
          for (final newLine in patch.replacement.split('\n')) {
            buffer.writeln('+ $newLine');
          }
          buffer.writeln(' ${lines[i]}');
        } else {
          buffer.writeln(' ${lines[i]}');
        }
      }
      offsetCounter += lines[i].length + 1;
    }

    buffer.writeln();
  }

  return buffer.toString();
}
