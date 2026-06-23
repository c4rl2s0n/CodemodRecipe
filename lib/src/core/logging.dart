import 'package:logging/logging.dart';

/// Centralized logging utility for the codemod_recipe package.
///
/// Provides consistent logging across all components with configurable levels.
class CodemodLogger {
  static final CodemodLogger _instance = CodemodLogger._internal();

  // Main logger instance
  final Logger _logger = Logger('codemod_recipe');

  // Subsystem loggers
  final Logger _yamlLogger = Logger('codemod_recipe.yaml');
  final Logger _astLogger = Logger('codemod_recipe.ast');
  final Logger _fileLogger = Logger('codemod_recipe.file');
  final Logger _runnerLogger = Logger('codemod_recipe.runner');

  /// Private constructor
  CodemodLogger._internal() {
    // Initialize logging configuration
    _configureLogging();
  }

  /// Factory constructor to get the singleton instance
  factory CodemodLogger() {
    return _instance;
  }

  /// Configure logging levels and handlers
  void _configureLogging() {
    // Enable hierarchical logging to allow setting levels on non-root loggers
    hierarchicalLoggingEnabled = true;

    // Set default log levels
    _logger.level = Level.INFO;
    _yamlLogger.level = Level.INFO;
    _astLogger.level = Level.INFO;
    _fileLogger.level = Level.INFO;
    _runnerLogger.level = Level.INFO;

    // Add console handler
    _logger.onRecord.listen((record) {
      // Format: [LEVEL] [LOGGER_NAME] message
      final timestamp = DateTime.now().toIso8601String();
      print(
        '[${record.level.name}] [$timestamp] [${record.loggerName}] ${record.message}',
      );

      if (record.error != null) {
        print('  Error: ${record.error}');
      }

      if (record.stackTrace != null) {
        print('  Stack: ${record.stackTrace}');
      }
    });
  }

  /// Set log level for all loggers
  void setLogLevel(Level level) {
    _logger.level = level;
    _yamlLogger.level = level;
    _astLogger.level = level;
    _fileLogger.level = level;
    _runnerLogger.level = level;
  }

  /// Set log level for a specific subsystem
  void setSubsystemLogLevel(String subsystem, Level level) {
    switch (subsystem) {
      case 'yaml':
        _yamlLogger.level = level;
        break;
      case 'ast':
        _astLogger.level = level;
        break;
      case 'file':
        _fileLogger.level = level;
        break;
      case 'runner':
        _runnerLogger.level = level;
        break;
      default:
        _logger.level = level;
    }
  }

  /// Main logger methods
  void debug(String message, [Object? error, StackTrace? stackTrace]) {
    _logger.log(Level.FINE, message, error, stackTrace);
  }

  void info(String message, [Object? error, StackTrace? stackTrace]) {
    _logger.log(Level.INFO, message, error, stackTrace);
  }

  void warning(String message, [Object? error, StackTrace? stackTrace]) {
    _logger.log(Level.WARNING, message, error, stackTrace);
  }

  void error(String message, [Object? error, StackTrace? stackTrace]) {
    _logger.log(Level.SEVERE, message, error, stackTrace);
  }

  /// YAML processing logger
  void yamlDebug(String message, [Object? error, StackTrace? stackTrace]) {
    _yamlLogger.log(Level.FINE, message, error, stackTrace);
  }

  void yamlInfo(String message, [Object? error, StackTrace? stackTrace]) {
    _yamlLogger.log(Level.INFO, message, error, stackTrace);
  }

  void yamlWarning(String message, [Object? error, StackTrace? stackTrace]) {
    _yamlLogger.log(Level.WARNING, message, error, stackTrace);
  }

  void yamlError(String message, [Object? error, StackTrace? stackTrace]) {
    _yamlLogger.log(Level.SEVERE, message, error, stackTrace);
  }

  /// AST processing logger
  void astDebug(String message, [Object? error, StackTrace? stackTrace]) {
    _astLogger.log(Level.FINE, message, error, stackTrace);
  }

  void astInfo(String message, [Object? error, StackTrace? stackTrace]) {
    _astLogger.log(Level.INFO, message, error, stackTrace);
  }

  void astWarning(String message, [Object? error, StackTrace? stackTrace]) {
    _astLogger.log(Level.WARNING, message, error, stackTrace);
  }

  void astError(String message, [Object? error, StackTrace? stackTrace]) {
    _astLogger.log(Level.SEVERE, message, error, stackTrace);
  }

  /// File operations logger
  void fileDebug(String message, [Object? error, StackTrace? stackTrace]) {
    _fileLogger.log(Level.FINE, message, error, stackTrace);
  }

  void fileInfo(String message, [Object? error, StackTrace? stackTrace]) {
    _fileLogger.log(Level.INFO, message, error, stackTrace);
  }

  void fileWarning(String message, [Object? error, StackTrace? stackTrace]) {
    _fileLogger.log(Level.WARNING, message, error, stackTrace);
  }

  void fileError(String message, [Object? error, StackTrace? stackTrace]) {
    _fileLogger.log(Level.SEVERE, message, error, stackTrace);
  }

  /// Runner/execution logger
  void runnerDebug(String message, [Object? error, StackTrace? stackTrace]) {
    _runnerLogger.log(Level.FINE, message, error, stackTrace);
  }

  void runnerInfo(String message, [Object? error, StackTrace? stackTrace]) {
    _runnerLogger.log(Level.INFO, message, error, stackTrace);
  }

  void runnerWarning(String message, [Object? error, StackTrace? stackTrace]) {
    _runnerLogger.log(Level.WARNING, message, error, stackTrace);
  }

  void runnerError(String message, [Object? error, StackTrace? stackTrace]) {
    _runnerLogger.log(Level.SEVERE, message, error, stackTrace);
  }
}

/// Global logger instance for easy access
final logger = CodemodLogger();
