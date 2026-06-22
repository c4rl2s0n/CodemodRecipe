import 'dart:io';
import 'package:test/test.dart';
import 'package:path/path.dart' as path;

import '../../lib/src/cli/ast_path_generator.dart';

void main() {
  group('AstPathGenerator CLI', () {
    const testFileName = 'test_cli_sample.dart';
    late File testFile;
    late String testFilePath;
    
    setUp(() {
      // Create a test Dart file
      const sampleCode = '''
class SampleClass {
  int sampleField = 42;
  
  void sampleMethod() {
    print("Hello, World!");
  }
  
  SampleClass();
}

void topLevelFunction() {
  print("Top level");
}
''';
      
      testFilePath = path.join(Directory.systemTemp.path, testFileName);
      testFile = File(testFilePath)
        ..createSync()
        ..writeAsStringSync(sampleCode);
    });
    
    tearDown(() {
      // Clean up test file
      if (testFile.existsSync()) {
        testFile.deleteSync();
      }
    });
    
    group('generateFromFile', () {
      test('generates path for class declaration', () async {
        // Offset pointing to "SampleClass" in class declaration
        const classOffset = 6; // "SampleClass" starts here
        
        final result = await _captureOutput(() async {
          await AstPathGenerator.generateFromFile(
            testFilePath,
            classOffset,
            outputFormat: 'text'
          );
        });
        
        expect(result.exitCode, 0);
        expect(result.stdout, contains('AST Path for offset $classOffset'));
        expect(result.stdout, contains('classDecl: "SampleClass"'));
        expect(result.stdout, contains('anchor: memberLast'));
      });

      test('generates path for method declaration', () async {
        // Offset pointing to "sampleMethod"
        final methodOffset = sampleCode.indexOf('sampleMethod');
        
        final result = await _captureOutput(() async {
          await AstPathGenerator.generateFromFile(
            testFilePath,
            methodOffset,
            outputFormat: 'text'
          );
        });
        
        expect(result.exitCode, 0);
        expect(result.stdout, contains('method: "sampleMethod"'));
        expect(result.stdout, contains('anchor: stmtLast'));
      });

      test('generates path for field declaration', () async {
        // Offset pointing to "sampleField"
        final fieldOffset = sampleCode.indexOf('sampleField');
        
        final result = await _captureOutput(() async {
          await AstPathGenerator.generateFromFile(
            testFilePath,
            fieldOffset,
            outputFormat: 'text'
          );
        });
        
        expect(result.exitCode, 0);
        expect(result.stdout, contains('field: "sampleField"'));
        expect(result.stdout, contains('anchor: memberLast'));
      });

      test('generates YAML output', () async {
        const classOffset = 6;
        
        final result = await _captureOutput(() async {
          await AstPathGenerator.generateFromFile(
            testFilePath,
            classOffset,
            outputFormat: 'yaml',
            recipeId: 'test_recipe'
          );
        });
        
        expect(result.exitCode, 0);
        expect(result.stdout, contains('dslVersion: 1'));
        expect(result.stdout, contains('id: test_recipe'));
        expect(result.stdout, contains('classDecl: "SampleClass"'));
        expect(result.stdout, contains('anchor: memberLast'));
      });

      test('generates JSON output', () async {
        const classOffset = 6;
        
        final result = await _captureOutput(() async {
          await AstPathGenerator.generateFromFile(
            testFilePath,
            classOffset,
            outputFormat: 'json'
          );
        });
        
        expect(result.exitCode, 0);
        expect(result.stdout, contains('"file"'));
        expect(result.stdout, contains('"offset"'));
        expect(result.stdout, contains('"path"'));
        expect(result.stdout, contains('"navigate"'));
      });

      test('handles invalid offset gracefully', () async {
        final result = await _captureOutput(() async {
          await AstPathGenerator.generateFromFile(
            testFilePath,
            10000, // Invalid offset
            outputFormat: 'text'
          );
        });
        
        expect(result.exitCode, 1);
        expect(result.stderr, contains('No AST node found at offset'));
      });

      test('handles non-existent file gracefully', () async {
        final result = await _captureOutput(() async {
          await AstPathGenerator.generateFromFile(
            'non_existent_file.dart',
            0,
            outputFormat: 'text'
          );
        });
        
        expect(result.exitCode, 1);
        expect(result.stderr, contains('Error reading file'));
      });
    });

    group('main function', () {
      test('shows help when no arguments provided', () async {
        final result = await _captureOutput(() async {
          await AstPathGenerator.main([]);
        });
        
        expect(result.exitCode, 1);
        expect(result.stdout, contains('Usage:'));
        expect(result.stdout, contains('AST Path Generator'));
      });

      test('shows help with --help flag', () async {
        final result = await _captureOutput(() async {
          await AstPathGenerator.main(['--help']);
        });
        
        expect(result.exitCode, 0);
        expect(result.stdout, contains('Usage:'));
        expect(result.stdout, contains('--format'));
        expect(result.stdout, contains('--recipe-id'));
      });

      test('shows error for invalid offset', () async {
        final result = await _captureOutput(() async {
          await AstPathGenerator.main([testFilePath, 'invalid']);
        });
        
        expect(result.exitCode, 1);
        expect(result.stderr, contains('Invalid offset'));
      });

      test('generates output with valid arguments', () async {
        const classOffset = 6;
        
        final result = await _captureOutput(() async {
          await AstPathGenerator.main([
            testFilePath,
            classOffset.toString(),
            '--format', 'yaml',
            '--recipe-id', 'cli_test'
          ]);
        });
        
        expect(result.exitCode, 0);
        expect(result.stdout, contains('id: cli_test'));
        expect(result.stdout, contains('classDecl: "SampleClass"'));
      });
    });
  });
}

/// Helper to capture stdout/stderr and exit code from a function.
class _CaptureResult {
  final String stdout;
  final String stderr;
  final int exitCode;
  
  _CaptureResult(this.stdout, this.stderr, this.exitCode);
}

Future<_CaptureResult> _captureOutput(Function func) async {
  final stdoutBuffer = StringBuffer();
  final stderrBuffer = StringBuffer();
  int exitCode = 0;
  
  try {
    // Redirect stdout/stderr
    final originalStdout = stdout;
    final originalStderr = stderr;
    
    stdout = _CaptureStream(stdoutBuffer);
    stderr = _CaptureStream(stderrBuffer);
    
    await func();
    
  } catch (e) {
    if (e is Exit) {
      exitCode = e.code;
    } else {
      stderrBuffer.writeln('Unexpected error: $e');
      exitCode = 1;
    }
  } finally {
    // Restore original streams
    stdout = _originalStdout;
    stderr = _originalStderr;
  }
  
  return _CaptureResult(
    stdoutBuffer.toString(),
    stderrBuffer.toString(),
    exitCode
  );
}

/// Simple stream that captures output to a StringBuffer.
class _CaptureStream implements IOSink {
  final StringBuffer _buffer;
  
  _CaptureStream(this._buffer);
  
  @override
  void write(Object? object) {
    _buffer.write(object);
  }
  
  @override
  void writeAll(Iterable objects, [String separator = ""]) {
    _buffer.writeAll(objects, separator);
  }
  
  @override
  void writeln([Object? object = ""]) {
    _buffer.writeln(object);
  }
  
  @override
  void writeCharCode(int charCode) {
    _buffer.writeCharCode(charCode);
  }
  
  @override
  void add(List<int> data) {
    _buffer.write(String.fromCharCodes(data));
  }
  
  @override
  void addError(Object error, [StackTrace? stackTrace]) {
    _buffer.writeln('Error: $error');
    if (stackTrace != null) {
      _buffer.writeln(stackTrace);
    }
  }
  
  @override
  Future get done => Future.value();
  
  @override
  Future flush() => Future.value();
  
  @override
  Future close() => Future.value();
  
  // Ignore other methods for testing purposes
  @override
  noSuchMethod(Invocation invocation) => null;
}

// Store original streams
final _originalStdout = stdout;
final _originalStderr = stderr;