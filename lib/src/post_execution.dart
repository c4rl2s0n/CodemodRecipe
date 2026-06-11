import 'dart:io';

import 'context.dart';
import 'operation.dart';
import 'step.dart';

/// Result passed to post-execution actions after apply.
class CodemodRunResult {
  /// File changes that were applied.
  final List<FileChange> changes;

  /// Creates a run result.
  const CodemodRunResult({required this.changes});

  /// Changed paths that should be formatted.
  List<String> get formattablePaths {
    return changes
        .where((change) => change.hasChanges && change.shouldFormat)
        .map((change) => change.path)
        .toSet()
        .toList();
  }
}

/// Action that runs after a recipe is successfully applied.
abstract class PostExecution implements CodemodStep {
  /// Runs this action.
  Future<void> run(CodemodContext context, CodemodRunResult result);
  @override
  List<PostExecution> get postExecution => [this];
}

/// Base class for post actions backed by an external process.
abstract class ProcessPostExecutionBase implements PostExecution {
  /// Creates a process-backed post action.
  const ProcessPostExecutionBase();

  /// Executable to run.
  String executable(CodemodContext context, CodemodRunResult result);

  /// Arguments passed to [executable].
  List<String> arguments(CodemodContext context, CodemodRunResult result);

  /// Working directory for the process.
  String workingDirectory(CodemodContext context, CodemodRunResult result) {
    return Directory.current.path;
  }

  /// Message used when the process exits with a non-zero status.
  String failureMessage(
    CodemodContext context,
    CodemodRunResult result,
    String executable,
    List<String> arguments,
  ) {
    return '$executable ${arguments.join(' ')} failed';
  }

  /// Whether the process should run.
  bool shouldRun(CodemodContext context, CodemodRunResult result) => true;

  @override
  Future<void> run(CodemodContext context, CodemodRunResult result) async {
    if (!shouldRun(context, result)) return;

    final executableValue = executable(context, result);
    final argumentsValue = arguments(context, result);
    final process = await Process.run(
      executableValue,
      argumentsValue,
      workingDirectory: workingDirectory(context, result),
    );

    stdout.write(process.stdout);
    stderr.write(process.stderr);

    if (process.exitCode != 0) {
      throw StateError(
        failureMessage(context, result, executableValue, argumentsValue),
      );
    }
  }
}
