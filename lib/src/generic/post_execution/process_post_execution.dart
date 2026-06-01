import '../../context.dart';
import '../../post_execution.dart';

/// Runs a fixed process after a codemod applies.
class ProcessPostExecution extends ProcessPostExecutionBase {
  final String _executable;
  final List<String> _arguments;
  final String? _workingDirectory;

  /// Creates a process post action.
  const ProcessPostExecution(
    this._executable,
    this._arguments, {
    String? workingDirectory,
  }) : _workingDirectory = workingDirectory;

  @override
  String executable(CodemodContext context, CodemodRunResult result) {
    return _executable;
  }

  @override
  List<String> arguments(CodemodContext context, CodemodRunResult result) {
    return _arguments;
  }

  @override
  String workingDirectory(CodemodContext context, CodemodRunResult result) {
    return _workingDirectory ?? super.workingDirectory(context, result);
  }
}
