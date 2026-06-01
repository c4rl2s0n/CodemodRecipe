/// Fluent API for Dart code modifications.
///
/// Provides a chainable interface for AST-guided code edits.
// ignore_for_file: dangling_library_doc_comments, deprecated_member_use

import 'package:analyzer/dart/ast/ast.dart';

import '../patch_helpers.dart';
import 'ast_helpers.dart';

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
  final List<SourcePatch> _patches = [];

  ClassDeclaration? _currentClass;

  CodeEditor._(this._source, this._unit);

  /// Parses [source] and creates an editor for collecting patches.
  ///
  /// Parsing uses the analyzer package without resolving imports or types.
  factory CodeEditor(String source) {
    return CodeEditor._(source, parseSource(source));
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

  /// Returns whether the selected class declares a field named [name].
  ///
  /// Only fields declared directly in the selected class are considered.
  bool hasField(String name) {
    if (_currentClass == null) {
      throw StateError('No class selected. Call inClass() first.');
    }
    return hasFieldInClass(_currentClass!, name);
  }

  /// Returns whether the selected class constructor has a parameter named [name].
  bool hasConstructorParam(String name) {
    if (_currentClass == null) {
      throw StateError('No class selected. Call inClass() first.');
    }

    final constructor = findConstructor(_currentClass!);
    if (constructor == null) return false;

    final paramsSource = _source.substring(
      constructor.parameters.offset,
      constructor.parameters.end,
    );
    return RegExp(
      r'(\bthis\.|\b)' + RegExp.escape(name) + r'\b',
    ).hasMatch(paramsSource);
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

  /// Adds a `final` field declaration to the selected class.
  ///
  /// When [addToConstructor] is true, also adds a `this.name` constructor
  /// parameter to the first unnamed constructor.
  CodeEditor addField(
    String name,
    String type, {
    String? defaultValue,
    bool addToConstructor = true,
  }) {
    if (_currentClass == null) {
      throw StateError('No class selected. Call inClass() first.');
    }

    final fields = getFields(_currentClass!);
    final fieldInsertOffset = fields.isNotEmpty
        ? fields.last.end
        : findClassBodyStartOffset(_currentClass!);

    _patches.add(
      SourcePatch(
        fieldInsertOffset,
        0,
        '\n\n  final $type $name;',
        description: 'Add field $name to ${_currentClass!.name.lexeme}',
      ),
    );

    if (addToConstructor) {
      final constructor = findConstructor(_currentClass!);
      if (constructor != null) {
        final paramOffset = findLastParameterOffset(constructor);

        final paramCode = StringBuffer();
        paramCode.writeln(',');
        paramCode.write('    this.$name');
        if (defaultValue != null) {
          paramCode.write(' = $defaultValue');
        }

        _patches.add(
          SourcePatch(
            paramOffset,
            0,
            paramCode.toString(),
            description: 'Add $name parameter to constructor',
          ),
        );
      }
    }

    return this;
  }

  /// Adds a field only when the selected class has no field named [name].
  ///
  /// This is the preferred helper for idempotent field transforms.
  CodeEditor addFieldUnlessExists(
    String name,
    String type, {
    String? defaultValue,
    bool addToConstructor = true,
  }) {
    if (!hasField(name)) {
      addField(
        name,
        type,
        defaultValue: defaultValue,
        addToConstructor: addToConstructor,
      );
    }
    return this;
  }

  /// Adds a parameter to the selected class's unnamed constructor.
  ///
  /// When [thisPrefix] is true, emits `this.name`; otherwise emits
  /// `type name`.
  CodeEditor addConstructorParam(
    String name,
    String type, {
    String? defaultValue,
    bool thisPrefix = true,
  }) {
    if (_currentClass == null) {
      throw StateError('No class selected. Call inClass() first.');
    }

    final constructor = findConstructor(_currentClass!);
    if (constructor == null) {
      throw StateError('No constructor found in ${_currentClass!.name.lexeme}');
    }

    final paramOffset = findLastParameterOffset(constructor);
    final hasParameters = constructor.parameters.parameters.isNotEmpty;

    final buffer = StringBuffer();
    if (hasParameters) {
      buffer.writeln(',');
      buffer.write('    ');
    }
    if (thisPrefix) {
      buffer.write('this.$name');
    } else {
      buffer.write('$type $name');
    }
    if (defaultValue != null) {
      buffer.write(' = $defaultValue');
    }

    _patches.add(
      SourcePatch(
        paramOffset,
        0,
        buffer.toString(),
        description: 'Add constructor parameter $name',
      ),
    );

    return this;
  }

  /// Adds a constructor parameter only when it is not already present.
  ///
  /// This is the preferred helper for idempotent constructor transforms.
  CodeEditor addConstructorParamUnlessExists(
    String name,
    String type, {
    String? defaultValue,
    bool thisPrefix = true,
  }) {
    if (!hasConstructorParam(name)) {
      addConstructorParam(
        name,
        type,
        defaultValue: defaultValue,
        thisPrefix: thisPrefix,
      );
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
