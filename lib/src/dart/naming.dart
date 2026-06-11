/// Converts a PascalCase identifier to camelCase.
String pascalToCamel(String pascal) {
  if (pascal.isEmpty) return pascal;
  return pascal[0].toLowerCase() + pascal.substring(1);
}

/// Converts a PascalCase identifier to snake_case.
String pascalToSnake(String pascal) {
  if (pascal.isEmpty) return pascal;

  final buffer = StringBuffer();
  for (var i = 0; i < pascal.length; i++) {
    final char = pascal[i];
    if (char.toUpperCase() == char && i > 0) {
      buffer.write('_');
    }
    buffer.write(char.toLowerCase());
  }
  return buffer.toString();
}

/// Converts a camelCase identifier to PascalCase.
String camelToPascal(String camel) {
  if (camel.isEmpty) return camel;
  return camel[0].toUpperCase() + camel.substring(1);
}

/// Converts a Dart-style identifier to snake_case.
///
/// Handles common PascalCase and camelCase inputs, including acronym
/// boundaries such as `URLValue`.
String toSnakeCase(String input) {
  if (input.isEmpty) return input;

  final buffer = StringBuffer();
  var previousWasUpper = false;

  for (var i = 0; i < input.length; i++) {
    final char = input[i];
    final isUpper = char.toUpperCase() == char;
    final isLower = char.toLowerCase() == char && !isUpper;

    if (isUpper) {
      if (i > 0 &&
          (!previousWasUpper ||
              (i + 1 < input.length &&
                  input[i + 1].toLowerCase() == input[i + 1]))) {
        buffer.write('_');
      }
      buffer.write(char.toLowerCase());
    } else if (isLower) {
      buffer.write(char);
    } else {
      buffer.write(char);
    }

    previousWasUpper = isUpper;
  }

  return buffer.toString();
}

/// Converts a Dart-style identifier to PascalCase.
String toPascalCase(String input) {
  if (input.isEmpty) return input;

  final words = _identifierWords(input);
  return words.map((word) => camelToPascal(word.toLowerCase())).join();
}

/// Converts a Dart-style identifier to camelCase.
String toCamelCase(String input) {
  final pascal = toPascalCase(input);
  return pascalToCamel(pascal);
}

List<String> _identifierWords(String input) {
  return toSnakeCase(
    input,
  ).split('_').where((part) => part.isNotEmpty).toList();
}
