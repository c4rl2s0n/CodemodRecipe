import 'dart:io';

import 'package:stubble/stubble.dart';

import 'context.dart';
import 'dart_codegen/naming.dart';

/// Renders inline or file-backed templates with codemod context values.
///
/// Templates use `stubble` syntax (Handlebars-like).
///
/// Supported helpers:
/// - `{{\$snake name}}`
/// - `{{\$camel name}}`
/// - `{{\$pascal name}}`
/// - `{{\$map "mapId" key}}` (see YAML maps)
///
/// ## Example
///
/// ```dart
/// final template = CodemodTemplate.inline('''
/// class {{\$pascal feature}}View {
///   final {{\$pascal feature}}Controller {{\$camel feature}}Controller;
/// }
/// ''');
///
/// final output = template.render(CodemodContext({'feature': 'FeedList'}));
/// ```
class CodemodTemplate {
  final String? _inlineSource;
  final String? _filePath;
  final TemplateEnvironment? _environment;

  /// Creates a template from an inline string.
  ///
  /// Use this for short generated snippets or small file templates declared
  /// directly in a recipe.
  const CodemodTemplate.inline(
    String source, {
    TemplateEnvironment? environment,
  }) : _inlineSource = source,
       _filePath = null,
       _environment = environment;

  /// Creates a template loaded from [path] when rendered.
  ///
  /// The file is read synchronously during [render]. Use this when generated
  /// files are too large to keep comfortably inline.
  const CodemodTemplate.file(String path, {TemplateEnvironment? environment})
    : _inlineSource = null,
      _filePath = path,
      _environment = environment;

  /// Renders this template with [context].
  String render(CodemodContext context) {
    final stubble = Stubble();
    final environment = _environment ?? const TemplateEnvironment();

    final data = Map<String, Object?>.from(context.toTemplateData());
    data['LBRACE'] = '{';

    stubble.registerHelper('snake', (List attrs, Function? fn) {
      if (attrs.isEmpty) return '';
      final value = attrs.first;
      if (value == null || value.toString() == 'null') {
        throw StateError('Required variable is not set');
      }
      return toSnakeCase(value.toString());
    });
    stubble.registerHelper('camel', (List attrs, Function? fn) {
      if (attrs.isEmpty) return '';
      final value = attrs.first;
      if (value == null || value.toString() == 'null') {
        throw StateError('Required variable is not set');
      }
      return toCamelCase(value.toString());
    });
    stubble.registerHelper('pascal', (List attrs, Function? fn) {
      if (attrs.isEmpty) return '';
      final value = attrs.first;
      if (value == null || value.toString() == 'null') {
        throw StateError('Required variable is not set');
      }
      return toPascalCase(value.toString());
    });
    stubble.registerHelper('map', (List attrs, Function? fn) {
      if (attrs.length < 2) return '';
      final mapIdValue = attrs[0];
      final keyValue = attrs[1];
      if (mapIdValue == null || keyValue == null) {
        throw StateError('Required variable is not set');
      }
      final mapId = mapIdValue.toString();
      final key = keyValue.toString();
      final map = environment.maps[mapId];
      if (map == null) return key;
      return map[key] ?? key;
    });

    stubble.registerHelper('when', (List attrs, Function? fn) {
      if (fn == null) return '';
      if (attrs.isEmpty) return '';
      final value = attrs.first;
      final isTrue = value == true || value.toString() == 'true';
      if (!isTrue) return '';
      return fn(<String, Object?>{});
    });

    final source = _escapeLiteralBraces(_source);
    _validateRequiredVariables(source, context);
    final compiled = stubble.compile(source);
    try {
      return compiled(data);
    } catch (error) {
      final message = error.toString();
      if (message.contains('Required variable is not set')) {
        throw StateError('Required variable is not set');
      }
      rethrow;
    }
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

  static String _escapeLiteralBraces(String source) {
    final buffer = StringBuffer();
    var i = 0;
    while (i < source.length) {
      final char = source[i];
      if (char == '{' && i + 1 < source.length && source[i + 1] == '{') {
        // Copy template tag literally until the closing braces.
        final start = i;
        i += 2;
        while (i + 1 < source.length) {
          if (source[i] == '}' && source[i + 1] == '}') {
            i += 2;
            break;
          }
          i++;
        }
        buffer.write(source.substring(start, i));
        continue;
      }

      if (char == '{') {
        buffer.write('{{LBRACE}}');
        i++;
        continue;
      }

      buffer.write(char);
      i++;
    }
    return buffer.toString();
  }

  static void _validateRequiredVariables(
    String source,
    CodemodContext context,
  ) {
    // Enforce fail-fast behavior when templates reference missing variables.
    //
    // We only validate simple variable tags like `{{name}}` or `{{a.b}}`.
    // Helpers, blocks, and partials are intentionally ignored.
    final regex = RegExp(
      r'\\{\\{\\s*([A-Za-z_]\\w*)(?:\\.[^\\s\\}]+)?\\s*\\}\\}',
    );
    for (final match in regex.allMatches(source)) {
      final name = match.group(1);
      if (name == null) continue;
      if (!context.has(name)) {
        throw StateError('Required variable \"$name\" is not set');
      }
      final value = context.get<Object>(name);
      if (value is String && value.isEmpty) {
        throw StateError('Required variable \"$name\" is not set');
      }
    }
  }
}

/// Template-time environment for a compiled recipe (YAML maps, etc).
class TemplateEnvironment {
  const TemplateEnvironment({this.maps = const {}});

  /// Global + per-recipe merged maps keyed by id.
  final Map<String, Map<String, String>> maps;
}
