import 'dart:io';

import 'package:mustache_template/mustache_template.dart';

import 'context.dart';
import 'dart/naming.dart';

/// Renders inline or file-backed templates with codemod context values.
///
/// Templates use Mustache-style placeholders and add package-defined casing
/// filters for common code generation paths.
///
/// Supported placeholders:
///
/// - `{{name}}`: raw context value
/// - `{{name:snake}}`: value converted to snake_case
/// - `{{name:camel}}`: value converted to camelCase
/// - `{{name:pascal}}`: value converted to PascalCase
///
/// Missing values fail with a [StateError] instead of rendering silently.
///
/// ## Example
///
/// ```dart
/// final template = CodemodTemplate.inline('''
/// class {{feature:pascal}}View {
///   final {{feature:pascal}}Controller {{feature:camel}}Controller;
/// }
/// ''');
///
/// final output = template.render(CodemodContext({'feature': 'FeedList'}));
/// ```
class CodemodTemplate {
  final String? _inlineSource;
  final String? _filePath;

  /// Creates a template from an inline string.
  ///
  /// Use this for short generated snippets or small file templates declared
  /// directly in a recipe.
  const CodemodTemplate.inline(String source)
    : _inlineSource = source,
      _filePath = null;

  /// Creates a template loaded from [path] when rendered.
  ///
  /// The file is read synchronously during [render]. Use this when generated
  /// files are too large to keep comfortably inline.
  const CodemodTemplate.file(String path)
    : _inlineSource = null,
      _filePath = path;

  /// Renders this template with [context].
  ///
  /// Throws when a referenced variable is missing, empty, or when Mustache
  /// encounters an unresolved placeholder.
  String render(CodemodContext context) {
    final source = _source;
    final prepared = _prepare(source, context);
    return Template(
      prepared.source,
      name: _filePath ?? '<inline>',
      lenient: false,
    ).renderString(prepared.values);
  }

  /// Raw template source, loaded from disk for file-backed templates.
  ///
  /// Editor integrations use this to render live previews while preserving
  /// placeholder origins for highlighting.
  String get source => _source;

  String get _source {
    final inlineSource = _inlineSource;
    if (inlineSource != null) return inlineSource;

    final filePath = _filePath;
    if (filePath == null) {
      throw StateError('Template has no source');
    }

    return File(filePath).readAsStringSync();
  }

  _PreparedTemplate _prepare(String source, CodemodContext context) {
    final values = <String, String>{...context.values};
    var index = 0;
    final rewritten = source.replaceAllMapped(
      RegExp(r'\{\{\s*([A-Za-z_]\w*)(?::(snake|camel|pascal))?\s*\}\}'),
      (match) {
        final name = match.group(1)!;
        final casing = match.group(2);
        final value = context.require(name);
        final rendered = casing == null
            ? value
            : _applyCasing(value, casing, variableName: name);
        final key = '__codemod_template_${name}_${casing ?? 'raw'}_${index++}';
        values[key] = rendered;
        return '{{$key}}';
      },
    );

    return _PreparedTemplate(rewritten, values);
  }

  String _applyCasing(
    String value,
    String casing, {
    required String variableName,
  }) {
    return switch (casing) {
      'snake' => toSnakeCase(value),
      'camel' => toCamelCase(value),
      'pascal' => toPascalCase(value),
      _ => throw StateError('Unsupported casing "$casing" for "$variableName"'),
    };
  }
}

class _PreparedTemplate {
  final String source;
  final Map<String, String> values;

  const _PreparedTemplate(this.source, this.values);
}
