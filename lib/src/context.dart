import 'dart_codegen/field_spec.dart';
import 'dart_codegen/naming.dart';
import 'template.dart';

/// Stores recipe arguments and generic naming helpers for a codemod run.
///
/// The context is populated from CLI arguments and provides methods to
/// access values and convert between naming conventions (snake_case,
/// camelCase, PascalCase).
///
/// ## Example
///
/// ```dart
/// final context = CodemodContext({'feature': 'UserProfile'});
///
/// // Get raw value
/// context.get('feature'); // 'UserProfile'
///
/// // Convert naming conventions
/// context.snake('feature'); // 'user_profile'
/// context.camel('feature'); // 'userProfile'
/// context.pascal('feature'); // 'UserProfile'
///
/// // Render templates
/// context.render('lib/{{feature:snake}}.dart'); // 'lib/user_profile.dart'
/// ```
///
/// ## Project Extensions
///
/// For project-specific helpers, create an extension on [CodemodContext]:
///
/// ```dart
/// extension MyProjectContext on CodemodContext {
///   String get featureName => require('feature');
///   String get featurePath => 'lib/features/${snake('feature')}';
///   String get featureModelFile => '$featurePath/${snake('feature')}_model.dart';
/// }
/// ```
class CodemodContext {
  final Map<String, String> _values;

  /// Project-wide code generation preferences for this run.
  final CodemodPreferences preferences;

  /// Creates a context seeded with optional argument [values].
  ///
  /// Values are copied into a new mutable map, so changes to the provided
  /// map after construction do not affect the context.
  CodemodContext([
    Map<String, String> values = const {},
    this.preferences = const CodemodPreferences(),
  ]) : _values = Map<String, String>.from(values);

  /// Returns an immutable view of all values currently available to recipes.
  ///
  /// Changes to the returned map do not affect the context. To modify values,
  /// use [set] instead.
  Map<String, String> get values => Map.unmodifiable(_values);

  /// Sets or replaces a named context value.
  ///
  /// Values set programmatically can be accessed by transforms just like
  /// CLI-provided arguments.
  ///
  /// ## Example
  ///
  /// ```dart
  /// final context = CodemodContext();
  /// context.set('timestamp', DateTime.now().toIso8601String());
  /// ```
  void set(String name, String value) => _values[name] = value;

  /// Returns the value for [name], or null when it has not been provided.
  ///
  /// Use this for optional arguments where a missing value is acceptable.
  /// For required arguments, use [require] instead.
  ///
  /// ## Example
  ///
  /// ```dart
  /// final outputDir = context.get('output') ?? 'lib/generated';
  /// ```
  String? get(String name) => _values[name];

  /// Returns whether [name] is available in this context.
  ///
  /// Returns true if the value exists, even when it is empty. Use [require]
  /// when empty values should be rejected.
  ///
  /// ## Example
  ///
  /// ```dart
  /// if (context.has('verbose')) {
  ///   print('Verbose mode enabled');
  /// }
  /// ```
  bool has(String name) => _values.containsKey(name);

  /// Returns the value for [name], throwing when it is missing or empty.
  ///
  /// Use this for required arguments that must be present for the codemod
  /// to function correctly.
  ///
  /// Throws a [StateError] if the value is null or empty.
  ///
  /// ## Example
  ///
  /// ```dart
  /// final filePath = context.require('file'); // Never null
  /// ```
  String require(String name) {
    final value = get(name);
    if (value == null || value.isEmpty) {
      throw StateError('Required variable "$name" is not set');
    }
    return value;
  }

  /// Returns the named value converted to snake_case.
  ///
  /// Handles various input formats (PascalCase, camelCase, existing snake_case)
  /// and intelligently inserts underscores at word boundaries.
  ///
  /// ## Example
  ///
  /// ```dart
  /// context.set('name', 'UserProfile');
  /// context.snake('name'); // 'user_profile'
  ///
  /// context.set('name', 'URLParser');
  /// context.snake('name'); // 'url_parser'
  /// ```
  String snake(String name) => toSnakeCase(require(name));

  /// Returns the named value converted from PascalCase to camelCase.
  ///
  /// The first character is lowercased while the rest remains unchanged.
  ///
  /// ## Example
  ///
  /// ```dart
  /// context.set('name', 'UserProfile');
  /// context.camel('name'); // 'userProfile'
  /// ```
  String camel(String name) => toCamelCase(require(name));

  /// Returns the named value converted from camelCase to PascalCase.
  ///
  /// The first character is uppercased while the rest remains unchanged.
  ///
  /// ## Example
  ///
  /// ```dart
  /// context.set('name', 'userProfile');
  /// context.pascal('name'); // 'UserProfile'
  /// ```
  String pascal(String name) => toPascalCase(require(name));

  /// Renders an inline codemod template with this context.
  ///
  /// Convenience method equivalent to:
  /// ```dart
  /// CodemodTemplate.inline(template).render(this)
  /// ```
  ///
  /// ## Example
  ///
  /// ```dart
  /// context.set('feature', 'UserProfile');
  /// context.render('lib/{{feature:snake}}.dart'); // 'lib/user_profile.dart'
  /// ```
  String render(String template) =>
      CodemodTemplate.inline(template).render(this);
}


/// Project-wide defaults for code generation behavior.
class CodemodPreferences {
  /// Style used when adding a parameter to an empty `()` constructor.
  final ConstructorParamStyle emptyConstructorStyle;

  /// Creates preferences with optional overrides.
  const CodemodPreferences({
    this.emptyConstructorStyle = ConstructorParamStyle.named,
  });
}

/// Backwards-compatible name for older codemod examples.
///
/// @deprecated Use [CodemodContext] instead.
typedef TemplateContext = CodemodContext;
