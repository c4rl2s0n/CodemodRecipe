import '../../dart_codegen/naming.dart' as naming;

/// String utility functions for common naming convention conversions.
///
/// Provides helper methods for converting between different naming conventions
/// commonly used in code generation and file naming.
class StringUtils {
  /// Converts a string to snake_case.
  ///
  /// Handles various input formats (PascalCase, camelCase, existing snake_case)
  /// and intelligently inserts underscores at word boundaries.
  ///
  /// ## Examples
  /// ```dart
  /// StringUtils.toSnakeCase('UserProfile'); // 'user_profile'
  /// StringUtils.toSnakeCase('URLParser');    // 'url_parser'
  /// StringUtils.toSnakeCase('already_snake'); // 'already_snake'
  /// ```
  static String toSnakeCase(String input) => naming.toSnakeCase(input);

  /// Converts a string from PascalCase to camelCase.
  ///
  /// The first character is lowercased while the rest remains unchanged.
  ///
  /// ## Examples
  /// ```dart
  /// StringUtils.toCamelCase('UserProfile'); // 'userProfile'
  /// StringUtils.toCamelCase('HTMLParser');   // 'htmlParser'
  /// ```
  static String toCamelCase(String input) => naming.toCamelCase(input);

  /// Converts a string from camelCase to PascalCase.
  ///
  /// The first character is uppercased while the rest remains unchanged.
  ///
  /// ## Examples
  /// ```dart
  /// StringUtils.toPascalCase('userProfile'); // 'UserProfile'
  /// StringUtils.toPascalCase('htmlParser');   // 'HtmlParser'
  /// ```
  static String toPascalCase(String input) => naming.toPascalCase(input);

  /// Converts a string to kebab-case.
  ///
  /// Similar to snake_case but uses hyphens instead of underscores.
  ///
  /// ## Examples
  /// ```dart
  /// StringUtils.toKebabCase('UserProfile'); // 'user-profile'
  /// StringUtils.toKebabCase('HTMLParser');   // 'html-parser'
  /// ```
  static String toKebabCase(String input) {
    return toSnakeCase(input).replaceAll('_', '-');
  }

  /// Converts a string to SCREAMING_SNAKE_CASE (uppercase snake case).
  ///
  /// Useful for constants and environment variables.
  ///
  /// ## Examples
  /// ```dart
  /// StringUtils.toScreamingSnakeCase('UserProfile'); // 'USER_PROFILE'
  /// StringUtils.toScreamingSnakeCase('apiKey');      // 'API_KEY'
  /// ```
  static String toScreamingSnakeCase(String input) {
    return toSnakeCase(input).toUpperCase();
  }

  /// Converts a string to Title Case.
  ///
  /// Capitalizes the first letter of each word, with words typically
  /// separated by spaces.
  ///
  /// ## Examples
  /// ```dart
  /// StringUtils.toTitleCase('user_profile'); // 'User Profile'
  /// StringUtils.toTitleCase('apiKey');      // 'Api Key'
  /// ```
  static String toTitleCase(String input) {
    return input
      .replaceAllMapped(RegExp(r'[A-Z][a-z]'), (match) => ' ${match.group(0)}')
      .replaceAllMapped(RegExp(r'[A-Z]+'), (match) => ' ${match.group(0)}')
      .replaceAll('_', ' ')
      .replaceAll('-', ' ')
      .split(' ')
      .map((word) => word.isNotEmpty ? '${word[0].toUpperCase()}${word.substring(1).toLowerCase()}' : '')
      .join(' ')
      .trim();
  }

  /// Converts a string to a safe filename.
  ///
  /// Removes or replaces characters that are not allowed in filenames
  /// on most operating systems.
  ///
  /// ## Examples
  /// ```dart
  /// StringUtils.toSafeFilename('User Profile'); // 'user_profile'
  /// StringUtils.toSafeFilename('file:name.txt'); // 'file_name.txt'
  /// ```
  static String toSafeFilename(String input) {
    return input
      .replaceAll(RegExp(r'[<>:"|?*\/]'), '_')
      .replaceAll(RegExp(r'\s+'), '_')
      .trim();
  }

  /// Capitalizes the first letter of a string.
  ///
  /// ## Examples
  /// ```dart
  /// StringUtils.capitalize('hello'); // 'Hello'
  /// StringUtils.capitalize('world');  // 'World'
  /// ```
  static String capitalize(String input) {
    if (input.isEmpty) return input;
    return '${input[0].toUpperCase()}${input.substring(1)}';
  }

  /// Converts a string to lowercase.
  ///
  /// Null-safe version that returns empty string for null input.
  ///
  /// ## Examples
  /// ```dart
  /// StringUtils.lowercase('Hello'); // 'hello'
  /// StringUtils.lowercase(null);    // ''
  /// ```
  static String lowercase(String? input) {
    return input?.toLowerCase() ?? '';
  }

  /// Converts a string to uppercase.
  ///
  /// Null-safe version that returns empty string for null input.
  ///
  /// ## Examples
  /// ```dart
  /// StringUtils.uppercase('hello'); // 'HELLO'
  /// StringUtils.uppercase(null);    // ''
  /// ```
  static String uppercase(String? input) {
    return input?.toUpperCase() ?? '';
  }
}