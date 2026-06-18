import 'package:analyzer/dart/ast/ast.dart';

import '../core/context.dart';
import '../dart_codegen/ast_helpers/ast_focus.dart';
import '../core/template.dart';
import 'interpreter.dart';
import 'model.dart';
import 'navigate_parser.dart';
import 'parser.dart';

/// Resolves a string from codemod context.
typedef StringResolver = String Function(CodemodContext context);

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
      throw StateError('Expected class focus, got ${focus.node.runtimeType}');
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
        match: step.match == null
            ? null
            : CodemodTemplate.inline(step.match!).render(context),
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
    return [for (final entry in value) _parseNavigateEntry(entry)];
  }

  if (value is Map) {
    return [_parseNavigateEntry(value)];
  }

  throw FormatException('Unsupported navigate value: $value');
}

NavigateStep _parseNavigateEntry(Object? entry) {
  return NavigateParser.parseEntry(entry);
}
