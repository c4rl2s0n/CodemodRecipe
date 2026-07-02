import 'dart:io';

import 'operation.dart';

/// Applies [changes] atomically: stage all writes, then commit; rollback on error.
Future<void> applyFileChangesAtomically(List<FileChange> changes) async {
  final planned = <_PlannedWrite>[];

  for (final change in changes) {
    if (!change.hasChanges) continue;

    final file = File(change.path);
    final existed = await file.exists();
    final original = existed ? await file.readAsString() : null;

    late final String content;
    if (change is PatchFileChange) {
      content = change.generate();
    } else if (change is CreateFileChange) {
      content = change.content;
    } else {
      throw StateError('Unsupported change type: ${change.runtimeType}');
    }

    planned.add(
      _PlannedWrite(
        path: change.path,
        content: content,
        originalContent: original,
        existedBefore: existed,
      ),
    );
  }

  if (planned.isEmpty) return;

  try {
    for (final write in planned) {
      final file = File(write.path);
      await file.parent.create(recursive: true);
      await file.writeAsString(write.content);
    }
  } catch (error) {
    for (final write in planned) {
      final file = File(write.path);
      if (!write.existedBefore) {
        if (await file.exists()) {
          await file.delete();
        }
        continue;
      }
      await file.writeAsString(write.originalContent!);
    }
    rethrow;
  }
}

class _PlannedWrite {
  const _PlannedWrite({
    required this.path,
    required this.content,
    required this.originalContent,
    required this.existedBefore,
  });

  final String path;
  final String content;
  final String? originalContent;
  final bool existedBefore;
}
