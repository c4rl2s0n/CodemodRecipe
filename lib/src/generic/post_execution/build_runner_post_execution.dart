import 'process_post_execution.dart';

/// Runs build_runner after a codemod applies.
class BuildRunnerPostExecution extends ProcessPostExecution {
  /// Creates a build_runner post action.
  BuildRunnerPostExecution()
    : super('dart', const [
        'run',
        'build_runner',
        'build',
        '--delete-conflicting-outputs',
      ]);
}
