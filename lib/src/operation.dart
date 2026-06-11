import 'dart:io';

import 'context.dart';
import 'patch_helpers.dart';
import 'step.dart';
import 'template.dart';
import 'transform.dart';

/// Resolves a target file path from recipe context.
typedef PathResolver = String Function(CodemodContext context);

/// Builds transforms to run for a target file.
typedef TransformResolver =
    List<CodeTransform> Function(CodemodContext context);

/// Describes what to do when a generated file already exists.
enum FileExistsStrategy {
  /// Fail the run if the file exists.
  fail,

  /// Leave the existing file unchanged.
  skip,

  /// Replace the existing file content.
  overwrite,
}

/// One recipe step that plans one or more file changes.
abstract class CodemodOperation with CodemodStep {
  /// Collects planned changes for this operation.
  Future<List<FileChange>> collect(CodemodContext context);
  @override
  List<CodemodOperation> get operations => [this];
}

/// A planned file change that can be previewed or applied.
abstract interface class FileChange {
  /// Path relative to the current working directory.
  String get path;

  /// Whether this change writes new file content.
  bool get hasChanges;

  /// Whether this path should be passed to `dart format`.
  bool get shouldFormat;

  /// Human-readable dry-run preview.
  String preview();

  /// Writes this change to disk.
  Future<void> apply();
}

/// Edits an existing Dart file with source patches from transforms.
class EditDartFileOperation extends CodemodOperation {
  final PathResolver path;
  final TransformResolver transforms;

  /// Creates an operation that edits an existing Dart file.
  EditDartFileOperation({required this.path, required this.transforms});

  @override
  Future<List<FileChange>> collect(CodemodContext context) async {
    final relativePath = path(context);
    final file = File(relativePath);

    if (!await file.exists()) {
      throw StateError('File not found: $relativePath');
    }

    final source = await file.readAsString();
    final patches = <SourcePatch>[];

    for (final transform in transforms(context)) {
      patches.addAll(await transform.apply(source, context));
    }

    validateNonOverlappingPatches(patches);

    return [
      PatchFileChange(path: relativePath, source: source, patches: patches),
    ];
  }
}

/// Creates a file from a rendered template.
class CreateFileOperation extends CodemodOperation {
  final PathResolver path;
  final CodemodTemplate template;
  final FileExistsStrategy ifExists;
  final bool format;
  final String? pathTemplate;
  final String? previewLabel;

  /// Creates a file operation.
  CreateFileOperation({
    required this.path,
    required this.template,
    this.ifExists = FileExistsStrategy.fail,
    this.format = true,
  }) : pathTemplate = null,
       previewLabel = null;

  /// Creates a file operation from a target path template.
  ///
  /// Editor integrations can reuse [pathTemplate] and [template] for live
  /// previews, avoiding duplicate recipe-level preview declarations.
  CreateFileOperation.templatePath({
    required this.pathTemplate,
    required this.template,
    this.ifExists = FileExistsStrategy.fail,
    this.format = true,
    this.previewLabel,
  }) : path = ((context) => context.render(pathTemplate!)),
       assert(pathTemplate != null);

  @override
  Future<List<FileChange>> collect(CodemodContext context) async {
    final relativePath = path(context);
    final file = File(relativePath);
    final exists = await file.exists();

    if (exists && ifExists == FileExistsStrategy.fail) {
      throw StateError('File already exists: $relativePath');
    }

    if (exists && ifExists == FileExistsStrategy.skip) {
      return [
        CreateFileChange.skip(
          path: relativePath,
          content: await file.readAsString(),
          shouldFormat: false,
        ),
      ];
    }

    return [
      CreateFileChange(
        path: relativePath,
        content: template.render(context),
        exists: exists,
        shouldFormat: format && relativePath.endsWith('.dart'),
      ),
    ];
  }
}

/// A patch-based change for an existing file.
class PatchFileChange implements FileChange {
  @override
  final String path;

  /// Original file source.
  final String source;

  /// Patches to apply to [source].
  final List<SourcePatch> patches;

  /// Creates a patch change.
  const PatchFileChange({
    required this.path,
    required this.source,
    required this.patches,
  });

  @override
  bool get hasChanges => patches.isNotEmpty;

  @override
  bool get shouldFormat => path.endsWith('.dart');

  /// Returns the generated source after applying [patches].
  String generate() => applyPatches(source, patches);

  @override
  String preview() {
    if (!hasChanges) return 'No changes.';

    final buffer = StringBuffer();
    for (final patch in patches) {
      buffer.writeln('  ${patch.description ?? 'Patch'}');
    }
    buffer.writeln();
    buffer.write(previewPatches(source, patches));
    return buffer.toString();
  }

  @override
  Future<void> apply() {
    return File(path).writeAsString(generate());
  }
}

/// A full-content change for a created or overwritten file.
class CreateFileChange implements FileChange {
  @override
  final String path;

  /// Content to write.
  final String content;

  /// Whether this write overwrites an existing file.
  final bool exists;

  @override
  final bool shouldFormat;

  final bool _skipped;

  /// Creates a generated file change.
  const CreateFileChange({
    required this.path,
    required this.content,
    required this.exists,
    this.shouldFormat = true,
  }) : _skipped = false;

  const CreateFileChange.skip({
    required this.path,
    required this.content,
    required this.shouldFormat,
  }) : exists = true,
       _skipped = true;

  @override
  bool get hasChanges => !_skipped;

  @override
  String preview() {
    if (_skipped) return 'Skipped because file already exists.';

    final action = exists ? 'Overwrite file' : 'Create file';
    return '$action with content:\n\n```\n$content\n```';
  }

  @override
  Future<void> apply() async {
    if (_skipped) return;

    final file = File(path);
    await file.parent.create(recursive: true);
    await file.writeAsString(content);
  }
}
