import 'dart:io';

import 'package:codemod_recipe/codemod_recipe.dart';
import 'package:codemod_recipe/codemod_recipe_vscode.dart';
import 'package:test/test.dart';

void main() {
  group('applyFileChangesAtomically', () {
    test('rolls back all files when a later write fails', () async {
      final dir = await Directory.systemTemp.createTemp('atomic_apply_');
      addTearDown(() => dir.deleteSync(recursive: true));

      final okFile = File('${dir.path}/ok.dart')
        ..writeAsStringSync('original ok\n');
      final badFile = File('${dir.path}/bad.dart')
        ..writeAsStringSync('original bad\n');

      final changes = <FileChange>[
        PatchFileChange(
          path: okFile.path,
          source: 'original ok\n',
          patches: [SourcePatch(0, 0, 'patched ')],
        ),
        _FailingFileChange(badFile.path),
      ];

      expect(
        () => applyFileChangesAtomically(changes),
        throwsA(isA<StateError>()),
      );
      expect(okFile.readAsStringSync(), 'original ok\n');
      expect(badFile.readAsStringSync(), 'original bad\n');
    });

    test('commits all files when every change succeeds', () async {
      final dir = await Directory.systemTemp.createTemp('atomic_apply_ok_');
      addTearDown(() => dir.deleteSync(recursive: true));

      final file = File('${dir.path}/counter.dart')
        ..writeAsStringSync('class Counter {}\n');

      await applyFileChangesAtomically([
        PatchFileChange(
          path: file.path,
          source: 'class Counter {}\n',
          patches: [SourcePatch(16, 0, '\n  int value = 0;\n')],
        ),
      ]);

      expect(file.readAsStringSync(), contains('int value = 0'));
    });
  });
}

class _FailingFileChange implements FileChange {
  _FailingFileChange(this.path);

  @override
  final String path;

  @override
  bool get hasChanges => true;

  @override
  bool get shouldFormat => false;

  @override
  String preview() => 'fail';

  @override
  Future<void> apply() async {
    await File(path).writeAsString('partial');
    throw StateError('simulated write failure');
  }
}
