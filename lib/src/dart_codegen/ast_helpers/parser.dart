import 'package:analyzer/dart/analysis/utilities.dart';
import 'package:analyzer/dart/ast/ast.dart';

/// Parses Dart [source] into an unresolved analyzer compilation unit.
CompilationUnit parseSource(String source, {String path = '<unknown>'}) {
  final result = parseString(content: source, path: path);
  return result.unit;
}
