import 'package:analyzer/dart/ast/ast.dart';

import '../context.dart';
import '../dart_codegen/ast_helpers/ast_focus.dart';
import '../generic/transforms/resolvers.dart';
import '../template.dart';
import 'interpreter.dart';
import 'model.dart';
import 'parser.dart';

/// Resolves a class [AstFocus] from optional navigation steps or [className].
AstFocus resolveClassFocus(
  String source,
  CodemodContext context, {
  List<NavigateStep>? navigate,
  StringResolver? className,
}) {
  if (navigate != null && navigate.isNotEmpty) {
    final renderedNavigate = _renderNavigateSteps(navigate, context);
    final focus = AstPathInterpreter().navigateTo(source, renderedNavigate);
    if (focus.node is! ClassDeclaration) {
      throw StateError(
        'Expected class focus, got ${focus.node.runtimeType}',
      );
    }
    return focus;
  }

  if (className == null) {
    throw StateError('className or navigate is required');
  }

  return AstFocus.parse(source).classNamed(className(context));
}

List<NavigateStep> _renderNavigateSteps(
  List<NavigateStep> steps,
  CodemodContext context,
) {
  return [
    for (final step in steps)
      NavigateStep(
        step.kind,
        name: step.name == null
            ? null
            : CodemodTemplate.inline(step.name!).render(context),
      ),
  ];
}

List<NavigateStep>? parseNavigateSteps(Object? value) {
  if (value == null) return null;
  if (value is String) {
    final atIndex = value.lastIndexOf('@');
    if (atIndex >= 0) {
      return parsePathString(value).navigate;
    }
    return parsePathString('$value @ member:last').navigate;
  }

  if (value is List) {
    return [
      for (final entry in value) _parseNavigateEntry(entry),
    ];
  }

  if (value is Map) {
    return [_parseNavigateEntry(value)];
  }

  throw FormatException('Unsupported navigate value: $value');
}

NavigateStep _parseNavigateEntry(Object? entry) {
  if (entry is String) {
    return _parseNavigateToken(entry);
  }
  if (entry is Map) {
    if (entry.length != 1) {
      throw FormatException('Navigate map must have one key: $entry');
    }
    final key = entry.keys.first.toString();
    final raw = entry.values.first;
    final name = raw == null ? null : raw.toString();
    return _navigateStepForKey(key, name);
  }
  throw FormatException('Unsupported navigate entry: $entry');
}

NavigateStep _parseNavigateToken(String token) {
  final trimmed = token.trim();
  if (trimmed.isEmpty) {
    throw FormatException('Navigate token must not be empty');
  }
  if (trimmed == '.' || trimmed == 'root') {
    return const NavigateStep(NavigateKind.root);
  }

  final colonIndex = trimmed.indexOf(':');
  if (colonIndex < 0) {
    return NavigateStep(NavigateKind.classDecl, name: trimmed);
  }

  final key = trimmed.substring(0, colonIndex);
  final value = trimmed.substring(colonIndex + 1).trim();
  return _navigateStepForKey(key, value.isEmpty ? null : value);
}

NavigateStep _navigateStepForKey(String key, String? name) {
  return switch (key) {
    'class' => NavigateStep(
      NavigateKind.classDecl,
      name: _requireName(name, key),
    ),
    'method' => NavigateStep(NavigateKind.method, name: _requireName(name, key)),
    'ctor' => NavigateStep(NavigateKind.constructor, name: name),
    'call' => NavigateStep(NavigateKind.call, name: _requireName(name, key)),
    'import' => NavigateStep(NavigateKind.import, name: _requireName(name, key)),
    _ => throw FormatException('Unknown navigate step "$key"'),
  };
}

String _requireName(String? name, String key) {
  if (name == null || name.isEmpty) {
    throw FormatException('Navigate step "$key" requires a name');
  }
  return name;
}
