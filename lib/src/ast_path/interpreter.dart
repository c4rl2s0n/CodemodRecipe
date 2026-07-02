import 'package:analyzer/dart/ast/ast.dart';

import '../dart_codegen/ast_helpers/ast_focus.dart';
import '../dart_codegen/ast_helpers/localizers.dart';
import '../dart_codegen/ast_helpers/planners.dart';
import 'anchors.dart';
import 'model.dart';

/// Function signature for AST navigation step handlers.
///
/// Takes the current focus and a navigation step, returns a new focus.
typedef NavigateStepHandler =
    AstFocus Function(AstFocus focus, NavigateStep step);

/// Thrown when navigation or anchor resolution fails at runtime.
class AstPathResolutionException implements Exception {
  /// Creates a resolution exception.
  AstPathResolutionException(this.message, {this.code});

  /// Human-readable failure description.
  final String message;

  /// Optional stable error code for tooling.
  final String? code;

  @override
  String toString() {
    if (code == null) return 'AstPathResolutionException: $message';
    return 'AstPathResolutionException($code): $message';
  }
}

/// Navigates Dart source using [AstPath] and resolves insertion offsets.
class AstPathInterpreter {
  /// Resolves [path] to a byte offset in [source].
  int resolveOffset(
    String source,
    AstPath path, {
    String filePath = '<unknown>',
  }) {
    return resolveSpan(source, path, filePath: filePath).offset;
  }

  /// Resolves [path] to a byte span in [source].
  AnchorSpan resolveSpan(
    String source,
    AstPath path, {
    String filePath = '<unknown>',
  }) {
    final focus = navigateTo(source, path.navigate, filePath: filePath);
    final node = focus.node;

    if (!isAnchorValidFor(node, path.anchor)) {
      throw AstPathResolutionException(
        "Anchor '${path.anchor}' invalid for focused node ${node.runtimeType}",
        code: 'E_ANCHOR_INVALID',
      );
    }

    return resolveAnchorSpan(source: source, node: node, anchor: path.anchor);
  }

  /// Navigates [source] using [steps] and returns the focused node.
  AstFocus navigateTo(
    String source,
    List<NavigateStep> steps, {
    String filePath = '<unknown>',
  }) {
    return _navigate(source, steps, filePath: filePath);
  }

  /// Resolves [path] to an insertion plan for [text] at the anchored offset.
  InsertionPlan resolveInsertionPlan(
    String source,
    AstPath path,
    String text, {
    String filePath = '<unknown>',
  }) {
    return InsertionPlan(
      offset: resolveOffset(source, path, filePath: filePath),
      text: text,
    );
  }

  AstFocus _navigate(
    String source,
    List<NavigateStep> steps, {
    required String filePath,
  }) {
    var focus = AstFocus.parse(source, path: filePath);

    for (final step in steps) {
      focus = _applyStep(focus, step);
    }

    return focus;
  }

  /// Map of step handlers for polymorphic dispatch.
  /// This replaces the switch statement with a more maintainable lookup.
  /// Each handler has a signature that matches the specific step type requirements.
  late final _stepHandlers =
      <NavigateKind, AstFocus Function(AstFocus, NavigateStep)>{
        NavigateKind.root: (focus, step) => focus,
        NavigateKind.classDecl: (focus, step) =>
            _classNamed(focus, step.name!, step.match),
        NavigateKind.method: (focus, step) =>
            _methodNamed(focus, step.name!, step.match),
        NavigateKind.constructor: (focus, step) =>
            _constructor(focus, step.name, step.match),
        NavigateKind.call: (focus, step) =>
            _call(focus, step.name!, step.match),
        NavigateKind.import: (focus, step) => _import(focus, step.name!),
        NavigateKind.field: (focus, step) =>
            _fieldNamed(focus, step.name!, step.match),
        NavigateKind.function: (focus, step) =>
            _functionNamed(focus, step.name!, step.match),
        NavigateKind.variable: (focus, step) =>
            _variableNamed(focus, step.name!, step.match),
        NavigateKind.initializer: (focus, step) =>
            _initializer(focus, step.match),
        NavigateKind.redirection: (focus, step) =>
            _redirection(focus, step.match),
      };

  AstFocus _applyStep(AstFocus focus, NavigateStep step) {
    // Handle type-inferred navigation (kind is null)
    if (step.kind == null) {
      return _findByName(focus, step.name!, step.match);
    }

    final handler = _stepHandlers[step.kind!];
    if (handler != null) {
      return handler(focus, step);
    }

    throw AstPathResolutionException(
      'No handler for navigation step kind: ${step.kind}',
      code: 'E_NAVIGATION_UNKNOWN',
    );
  }

  AstFocus _classNamed(AstFocus focus, String name, String? match) {
    try {
      final candidates = findClassesByName(focus.unit, name);
      if (candidates.isEmpty) {
        throw StateError('Class "$name" not found in source');
      }

      final classDecl = _selectMatch(
        candidates,
        focus.source,
        match,
        label: 'class "$name"',
      );
      return AstFocus(focus.source, focus.unit, classDecl);
    } on StateError catch (error) {
      throw AstPathResolutionException(error.message, code: 'E_NODE_NOT_FOUND');
    }
  }

  AstFocus _methodNamed(AstFocus focus, String name, String? match) {
    try {
      final classDecl = focus.asClass;
      final methods = classMembers(classDecl)
          .whereType<MethodDeclaration>()
          .where((method) => methodNameLexeme(method) == name)
          .toList();
      if (methods.isEmpty) {
        throw StateError(
          'Method "$name" not found in ${classNameLexeme(classDecl)}',
        );
      }

      final method = _selectMatch(
        methods,
        focus.source,
        match,
        label: 'method "$name"',
      );
      return AstFocus(focus.source, focus.unit, method);
    } on StateError catch (error) {
      throw AstPathResolutionException(error.message, code: 'E_NODE_NOT_FOUND');
    }
  }

  AstFocus _constructor(AstFocus focus, String? name, String? match) {
    try {
      final classDecl = focus.asClass;
      final constructors = classMembers(classDecl)
          .whereType<ConstructorDeclaration>()
          .where((ctor) {
            final ctorName = ctor.name?.lexeme;
            if (name == null) {
              return ctorName == null;
            }
            return ctorName == name;
          })
          .toList();

      if (constructors.isEmpty && name == null) {
        final fallback = classMembers(classDecl)
            .whereType<ConstructorDeclaration>()
            .toList();
        if (fallback.length == 1) {
          return AstFocus(focus.source, focus.unit, fallback.first);
        }
      }

      if (constructors.isEmpty) {
        final label = name == null
            ? 'unnamed constructor'
            : 'constructor "$name"';
        throw StateError('$label not found in ${classNameLexeme(classDecl)}');
      }

      final ctor = _selectMatch(
        constructors,
        focus.source,
        match,
        label: name == null ? 'constructor' : 'constructor "$name"',
      );
      return AstFocus(focus.source, focus.unit, ctor);
    } on StateError catch (error) {
      throw AstPathResolutionException(error.message, code: 'E_NODE_NOT_FOUND');
    }
  }

  AstFocus _fieldNamed(AstFocus focus, String name, String? match) {
    try {
      final classDecl = focus.asClass;
      final fields = getFields(classDecl)
          .where(
            (field) => field.fields.variables.any(
              (variable) => variable.name.lexeme == name,
            ),
          )
          .toList();

      if (fields.isEmpty) {
        throw StateError('Field "$name" not found in ${classNameLexeme(classDecl)}');
      }

      final field = _selectMatch(
        fields,
        focus.source,
        match,
        label: 'field "$name"',
      );
      return AstFocus(focus.source, focus.unit, field);
    } on StateError catch (error) {
      throw AstPathResolutionException(error.message, code: 'E_NODE_NOT_FOUND');
    }
  }

  AstFocus _call(AstFocus focus, String typeName, String? match) {
    try {
      final call = focus.instanceCreation(typeName);
      if (match != null && !_nodeMatches(focus.source, call.node, match)) {
        throw StateError('$typeName(...) not found matching "$match"');
      }
      return call;
    } on StateError catch (error) {
      throw AstPathResolutionException(error.message, code: 'E_NODE_NOT_FOUND');
    }
  }

  AstFocus _import(AstFocus focus, String uri) {
    final imports = focus.unit.directives.whereType<ImportDirective>();
    for (final directive in imports) {
      if (directive.uri.stringValue == uri) {
        return AstFocus(focus.source, focus.unit, directive);
      }
    }

    throw AstPathResolutionException(
      'Import "$uri" not found',
      code: 'E_NODE_NOT_FOUND',
    );
  }

  /// Finds a node by name using type inference.
  /// Searches in order of priority: class > constructor > method > field > variable > function
  AstFocus _findByName(AstFocus focus, String name, String? match) {
    final node = focus.node;

    // If we're inside a class, search class members first
    if (node is ClassDeclaration) {
      // Check constructors
      final constructors = classMembers(node)
          .whereType<ConstructorDeclaration>()
          .where((ctor) => ctor.name?.lexeme == name)
          .toList();
      if (constructors.isNotEmpty) {
        final ctor = _selectMatch(
          constructors,
          focus.source,
          match,
          label: 'constructor "$name"',
        );
        return AstFocus(focus.source, focus.unit, ctor);
      }

      // Check methods
      final methods = classMembers(node)
          .whereType<MethodDeclaration>()
          .where((method) => methodNameLexeme(method) == name)
          .toList();
      if (methods.isNotEmpty) {
        final method = _selectMatch(
          methods,
          focus.source,
          match,
          label: 'method "$name"',
        );
        return AstFocus(focus.source, focus.unit, method);
      }

      // Check fields
      final fields = getFields(node)
          .where(
            (field) => field.fields.variables.any(
              (variable) => variable.name.lexeme == name,
            ),
          )
          .toList();
      if (fields.isNotEmpty) {
        final field = _selectMatch(
          fields,
          focus.source,
          match,
          label: 'field "$name"',
        );
        return AstFocus(focus.source, focus.unit, field);
      }
    }

    // Search at compilation unit level
    if (node is CompilationUnit || node == focus.unit) {
      final unit = focus.unit;

      // Check classes
      final classes = findClassesByName(unit, name);
      if (classes.isNotEmpty) {
        final classDecl = _selectMatch(
          classes,
          focus.source,
          match,
          label: 'class "$name"',
        );
        return AstFocus(focus.source, focus.unit, classDecl);
      }

      // Check top-level functions
      final functions = unit.declarations
          .whereType<FunctionDeclaration>()
          .where((func) => func.name.lexeme == name)
          .toList();
      if (functions.isNotEmpty) {
        final func = _selectMatch(
          functions,
          focus.source,
          match,
          label: 'function "$name"',
        );
        return AstFocus(focus.source, focus.unit, func);
      }

      // Check top-level variables
      final variables = unit.declarations
          .whereType<TopLevelVariableDeclaration>()
          .where(
            (decl) => decl.variables.variables.any(
              (variable) => variable.name.lexeme == name,
            ),
          )
          .toList();
      if (variables.isNotEmpty) {
        final varDecl = _selectMatch(
          variables,
          focus.source,
          match,
          label: 'variable "$name"',
        );
        return AstFocus(focus.source, focus.unit, varDecl);
      }
    }

    throw AstPathResolutionException(
      'Node with name "$name" not found',
      code: 'E_NODE_NOT_FOUND',
    );
  }

  AstFocus _functionNamed(AstFocus focus, String name, String? match) {
    try {
      final functions = focus.unit.declarations
          .whereType<FunctionDeclaration>()
          .where((func) => func.name.lexeme == name)
          .toList();

      if (functions.isEmpty) {
        throw StateError('Function "$name" not found in source');
      }

      final func = _selectMatch(
        functions,
        focus.source,
        match,
        label: 'function "$name"',
      );
      return AstFocus(focus.source, focus.unit, func);
    } on StateError catch (error) {
      throw AstPathResolutionException(error.message, code: 'E_NODE_NOT_FOUND');
    }
  }

  AstFocus _variableNamed(AstFocus focus, String name, String? match) {
    try {
      final variables = focus.unit.declarations
          .whereType<TopLevelVariableDeclaration>()
          .where(
            (decl) => decl.variables.variables.any(
              (variable) => variable.name.lexeme == name,
            ),
          )
          .toList();

      if (variables.isEmpty) {
        throw StateError('Variable "$name" not found in source');
      }

      final varDecl = _selectMatch(
        variables,
        focus.source,
        match,
        label: 'variable "$name"',
      );
      return AstFocus(focus.source, focus.unit, varDecl);
    } on StateError catch (error) {
      throw AstPathResolutionException(error.message, code: 'E_NODE_NOT_FOUND');
    }
  }

  AstFocus _initializer(AstFocus focus, String? match) {
    try {
      final constructor = focus.node;
      if (constructor is! ConstructorDeclaration) {
        throw StateError('initializer navigation requires a constructor');
      }

      // For now, just return the constructor itself
      // The anchor resolution will handle finding the initializer list
      return focus;
    } on StateError catch (error) {
      throw AstPathResolutionException(
        error.message,
        code: 'E_NAVIGATION_INVALID',
      );
    }
  }

  AstFocus _redirection(AstFocus focus, String? match) {
    try {
      final constructor = focus.node;
      if (constructor is! ConstructorDeclaration) {
        throw StateError('redirection navigation requires a constructor');
      }

      // Check if this is a redirecting factory constructor
      if (constructor.redirectedConstructor != null) {
        return AstFocus(
          focus.source,
          focus.unit,
          constructor.redirectedConstructor!,
        );
      }

      // For now, just return the constructor itself
      // The anchor resolution will handle finding the redirection
      return focus;
    } on StateError catch (error) {
      throw AstPathResolutionException(
        error.message,
        code: 'E_NAVIGATION_INVALID',
      );
    }
  }

  T _selectMatch<T extends AstNode>(
    List<T> candidates,
    String source,
    String? match, {
    required String label,
  }) {
    if (match == null) {
      if (candidates.length > 1) {
        throw StateError('Multiple $label matches; add a "match" filter');
      }
      return candidates.first;
    }

    final filtered = candidates
        .where((node) => _nodeMatches(source, node, match))
        .toList();
    if (filtered.isEmpty) {
      throw StateError('$label not found matching "$match"');
    }
    if (filtered.length > 1) {
      throw StateError('Multiple $label matches for "$match"');
    }
    return filtered.first;
  }

  bool _nodeMatches(String source, AstNode node, String match) {
    final snippet = source.substring(node.offset, node.end);
    return snippet.contains(match);
  }
}
