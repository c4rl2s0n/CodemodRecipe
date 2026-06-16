import 'package:analyzer/dart/ast/ast.dart';

import '../field_spec.dart';
import 'checkers.dart';
import 'invocations.dart';
import 'localizers.dart';
import 'offsets.dart';
import 'parser.dart';
import 'planners.dart';

/// Immutable cursor for navigating Dart source AST nodes.
///
/// Use [AstFocus.parse] to start from a compilation unit, chain navigation
/// methods to narrow the focus, then read offsets or insertion plans.
class AstFocus {
  /// Creates a focus on [node] within [source] and [unit].
  const AstFocus(this.source, this.unit, this.node);

  /// Original source text.
  final String source;

  /// Parsed compilation unit for [source].
  final CompilationUnit unit;

  /// Currently focused AST node.
  final AstNode node;

  /// Parses [source] and focuses on the compilation unit root.
  factory AstFocus.parse(String source, {String path = '<unknown>'}) {
    final parsedUnit = parseSource(source, path: path);
    return AstFocus(source, parsedUnit, parsedUnit);
  }

  /// Focused node as a [ClassDeclaration].
  ClassDeclaration get asClass => _require<ClassDeclaration>('class');

  /// Focused node as a [MethodDeclaration].
  MethodDeclaration get asMethod => _require<MethodDeclaration>('method');

  /// Focused node as a [ConstructorDeclaration].
  ConstructorDeclaration get asConstructor =>
      _require<ConstructorDeclaration>('constructor');

  /// Argument list when focused on a constructor-like call.
  ArgumentList get argumentList => argumentListOf(node);

  /// Whether this focus references a concrete node.
  bool get hasFocus => true;

  /// Focuses on the class named [name].
  AstFocus classNamed(String name) {
    final classDecl = findClassByName(unit, name);
    if (classDecl == null) {
      throw StateError('Class "$name" not found in source');
    }
    return AstFocus(source, unit, classDecl);
  }

  /// Focuses on the method or getter named [name] in the focused class.
  AstFocus methodNamed(String name) {
    final classDecl = asClass;
    final method = findMethodByName(classDecl, name);
    if (method == null) {
      throw StateError('Method "$name" not found in ${classDecl.name.lexeme}');
    }
    return AstFocus(source, unit, method);
  }

  /// Focuses on a constructor in the focused class.
  AstFocus constructor({String? name}) {
    final classDecl = asClass;
    final ctor = findConstructor(classDecl, name: name);
    if (ctor == null) {
      final label = name == null
          ? 'unnamed constructor'
          : 'constructor "$name"';
      throw StateError('$label not found in ${classDecl.name.lexeme}');
    }
    return AstFocus(source, unit, ctor);
  }

  /// Focuses on the field named [name] in the focused class.
  AstFocus fieldNamed(String name) {
    final classDecl = asClass;
    final field = findFieldByName(classDecl, name);
    if (field == null) {
      throw StateError('Field "$name" not found in ${classDecl.name.lexeme}');
    }
    return AstFocus(source, unit, field);
  }

  /// Focuses on a constructor-like call named [typeName] under the current node.
  ///
  /// Matches [InstanceCreationExpression] and unresolved [MethodInvocation]
  /// calls such as `Settings(...)`.
  AstFocus instanceCreation(
    String typeName, {
    bool returnExpressionOnly = false,
  }) {
    final call = findConstructorCall(
      node,
      typeName,
      returnExpressionOnly: returnExpressionOnly,
    );
    if (call == null) {
      throw StateError('$typeName(...) not found');
    }
    return AstFocus(source, unit, call);
  }

  /// Byte offset after the last argument in a focused constructor call.
  int get afterLastArgument =>
      findLastArgumentInsertOffset(source, argumentList);

  /// Byte offset after the last parameter in a focused constructor declaration.
  int get afterLastConstructorParameter =>
      findLastParameterOffset(asConstructor);

  /// Byte offset after the last statement in a focused method declaration.
  int get afterLastStatement => findLastStatementInsertOffset(asMethod);

  /// Plans insertion of [text] as a named argument in a focused constructor call.
  InsertionPlan planArgument(String text, {String indent = '      '}) {
    return planArgumentInsertion(source, argumentList, text, indent: indent);
  }

  /// Plans insertion of [text] into a focused constructor parameter list.
  InsertionPlan planConstructorParam(String text, ConstructorParamStyle kind) {
    return planConstructorParamInsertion(asConstructor, source, kind, text);
  }

  /// Whether the focused class declares [name].
  bool classHasMethod(String name) => hasMethodInClass(asClass, name);

  /// Whether the focused class declares field [name].
  bool classHasField(String name) => hasFieldInClass(asClass, name);

  /// Whether the focused constructor call has named argument [name].
  bool argumentsHaveNamed(String name) => hasNamedArgument(argumentList, name);

  T _require<T extends AstNode>(String label) {
    final focused = node;
    if (focused is! T) {
      throw StateError('Expected $label, got ${focused.runtimeType}');
    }
    return focused;
  }
}
