/// Fluent API for Dart code modifications.
///
/// Provides a chainable interface for AST-guided code edits.
// ignore_for_file: dangling_library_doc_comments, deprecated_member_use

import 'package:analyzer/dart/ast/ast.dart';

import '../patch_helpers.dart';
import 'ast_helpers.dart';

part 'editing/field_editor.dart';

/// Fluent editor for collecting AST-guided Dart source patches.
///
/// `CodeEditor` parses a Dart source string once and exposes chainable helpers
/// for common class-level modifications. The editor only collects
/// [SourcePatch]es; call [generate] or [applyPatches] to produce the modified
/// source.
///
/// ## Example
///
/// ```dart
/// final editor = CodeEditor(source)
///     .inClass('Counter')
///     .addMethodUnlessExists('increment', '''
///   void increment() {
///     value++;
///   }
/// ''');
///
/// final generated = editor.generate();
/// ```
class CodeEditor {
  final String _source;
  final CompilationUnit _unit;
  final CodemodPreferences _preferences;
  final List<SourcePatch> _patches = [];

  ClassDeclaration? _currentClass;

  CodeEditor._(this._source, this._unit, this._preferences);

  /// Parses [source] and creates an editor for collecting patches.
  ///
  /// Parsing uses the analyzer package without resolving imports or types.
  factory CodeEditor(
    String source, {
    CodemodPreferences preferences = const CodemodPreferences(),
  }) {
    return CodeEditor._(source, parseSource(source), preferences);
  }

  /// Selects the class named [name] as the target for subsequent operations.
  ///
  /// Subsequent helpers such as [addMethod] and [addField] operate on this
  /// selected class.
  ///
  /// Throws when no matching class declaration is found.
  CodeEditor inClass(String name) {
    final classDecl = findClassByName(_unit, name);
    if (classDecl == null) {
      throw StateError('Class "$name" not found in source');
    }
    _currentClass = classDecl;
    return this;
  }

  /// Returns whether the selected class declares a method named [name].
  ///
  /// Only methods declared directly in the selected class are considered.
  bool hasMethod(String name) {
    if (_currentClass == null) {
      throw StateError('No class selected. Call inClass() first.');
    }
    return hasMethodInClass(_currentClass!, name);
  }

  

  /// Adds [code] as a method-like class member near existing members.
  ///
  /// The code should include the desired indentation and method body. The
  /// editor inserts a blank line before the member.
  CodeEditor addMethod(String code) {
    if (_currentClass == null) {
      throw StateError('No class selected. Call inClass() first.');
    }

    final insertOffset = findOptimalInsertionOffset(_currentClass!);

    _patches.add(
      SourcePatch(
        insertOffset,
        0,
        '\n\n$code',
        description: 'Add method to ${_currentClass!.name.lexeme}',
      ),
    );

    return this;
  }

  /// Adds [code] only when the selected class has no method named [name].
  ///
  /// This is the preferred helper for idempotent transforms.
  CodeEditor addMethodUnlessExists(String name, String code) {
    if (!hasMethod(name)) {
      addMethod(code);
    }
    return this;
  }


  /// Returns all patches accumulated so far.
  ///
  /// The returned list is immutable.
  List<SourcePatch> get patches => List.unmodifiable(_patches);

  /// Generates the modified source code with all patches applied.
  ///
  /// This does not write to disk.
  String generate() => applyPatches(_source, _patches);

  /// Returns true if any patches have been generated.
  bool get hasChanges => _patches.isNotEmpty;

  /// Returns the number of patches.
  int get changeCount => _patches.length;

}
