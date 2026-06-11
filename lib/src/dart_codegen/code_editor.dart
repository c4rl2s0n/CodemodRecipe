/// Fluent API for Dart code modifications.
///
/// Provides a chainable interface for AST-guided code edits.
// ignore_for_file: dangling_library_doc_comments, deprecated_member_use

import 'package:analyzer/dart/ast/ast.dart';

import '../context.dart';
import '../patch_helpers.dart';
import 'ast_helpers/ast_helpers.dart';
import 'field_spec.dart';

export 'field_spec.dart';

part 'editing/field_editor.dart';

/// Collects [SourcePatch]es for AST-guided Dart source edits.
///
/// Use [AstFocus] for navigation and this editor for patching. Call
/// [generate] or read [patches] to produce the modified source.
///
/// ## Example
///
/// ```dart
/// final focus = AstFocus.parse(source).classNamed('Counter');
/// final generated = CodeEditor(source)
///     .addMethodUnlessExists(focus, 'increment', '''
///   void increment() {
///     value++;
///   }
/// ''')
///     .generate();
/// ```
class CodeEditor {
  final String _source;
  final CompilationUnit _unit;
  final CodemodPreferences _preferences;
  final List<SourcePatch> _patches = [];

  CodeEditor._(this._source, this._unit, this._preferences);

  /// Parses [source] and creates an editor for collecting patches.
  factory CodeEditor(
    String source, {
    CodemodPreferences preferences = const CodemodPreferences(),
  }) {
    return CodeEditor._(source, parseSource(source), preferences);
  }

  /// The original source string.
  String get source => _source;

  /// Parsed compilation unit for [source].
  CompilationUnit get unit => _unit;

  /// Root navigation focus for [source].
  AstFocus get root => AstFocus(_source, _unit, _unit);

  /// Inserts [text] at [offset].
  CodeEditor insert(int offset, String text, {String? description}) {
    _patches.add(
      SourcePatch(offset, 0, text, description: description),
    );
    return this;
  }

  /// Applies an [InsertionPlan] as a patch.
  CodeEditor insertPlan(InsertionPlan plan, {String? description}) {
    _patches.add(
      SourcePatch(
        plan.offset,
        plan.length,
        plan.text,
        description: description,
      ),
    );
    return this;
  }

  /// Adds [code] as a method-like class member near existing members.
  CodeEditor addMethod(AstFocus focus, String code) {
    final classDecl = focus.asClass;

    insert(
      findOptimalInsertionOffset(classDecl),
      '\n\n$code',
      description: 'Add method to ${classDecl.name.lexeme}',
    );

    return this;
  }

  /// Adds [code] only when the class has no method named [name].
  CodeEditor addMethodUnlessExists(
    AstFocus focus,
    String name,
    String code,
  ) {
    if (!focus.classHasMethod(name)) {
      addMethod(focus, code);
    }
    return this;
  }

  /// Returns all patches accumulated so far.
  List<SourcePatch> get patches => List.unmodifiable(_patches);

  /// Generates the modified source code with all patches applied.
  String generate() => applyPatches(_source, _patches);

  /// Returns true if any patches have been generated.
  bool get hasChanges => _patches.isNotEmpty;

  /// Returns the number of patches.
  int get changeCount => _patches.length;

  CodemodPreferences get preferences => _preferences;
}
