import '../../context.dart';
import '../../post_execution.dart';

/// Runs `dart format` on changed Dart files.
class DartFormatPostExecution extends ProcessPostExecutionBase {
  /// Creates a Dart format post action.
  DartFormatPostExecution();

  @override
  String executable(CodemodContext context, CodemodRunResult result) => 'dart';

  @override
  List<String> arguments(CodemodContext context, CodemodRunResult result) {
    return ['format', ...result.formattablePaths];
  }

  @override
  bool shouldRun(CodemodContext context, CodemodRunResult result) {
    return result.formattablePaths.isNotEmpty;
  }
}
