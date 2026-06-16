/// Converts a stored context value to its wire-format string.
///
/// Mirrors [ArgCodec] serialization rules for template rendering.
String stringifyArgValue(Object value) {
  return switch (value) {
    String s => s,
    bool b => b ? 'true' : 'false',
    Enum e => e.name,
    _ => value.toString(),
  };
}

/// Converts between typed recipe argument values and wire-format strings.
///
/// Supported value types: [String], [bool], [int], [double], and [Enum] subtypes.
abstract class ArgCodec<T extends Object> {
  const ArgCodec();

  /// Encodes [value] for CLI, host JSON, and [CodemodContext].
  String serialize(T value);

  /// Decodes a wire value. Returns null when [raw] is missing or invalid.
  T? parse(String? raw);

  /// Returns the built-in codec for [T].
  ///
  /// For enum types, pass [enumValues] (e.g. `MyMode.values`).
  static ArgCodec<T> of<T extends Object>({List<T>? enumValues}) {
    if (T == String) {
      return const StringArgCodec() as ArgCodec<T>;
    }
    if (T == bool) {
      return const BoolArgCodec() as ArgCodec<T>;
    }
    if (T == int) {
      return const IntArgCodec() as ArgCodec<T>;
    }
    if (T == double) {
      return const DoubleArgCodec() as ArgCodec<T>;
    }
    if (<T>[] is List<Enum>) {
      throw ArgumentError('enumValues is required for enum argument type $T');
    }
    throw ArgumentError('Unsupported CodemodArg type: $T');
  }

  /// Returns a codec for enum argument type [T].
  static ArgCodec<T> forEnum<T extends Enum>(List<T> values) {
    return EnumArgCodec(values);
  }

  /// Returns a codec backed by enum [values] at runtime.
  static ArgCodec<T> forEnumValues<T extends Object>(List<T> values) {
    return _RuntimeEnumArgCodec(values);
  }
}

class _RuntimeEnumArgCodec<T extends Object> extends ArgCodec<T> {
  const _RuntimeEnumArgCodec(this.values);

  final List<T> values;

  @override
  String serialize(T value) => (value as Enum).name;

  @override
  T? parse(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    for (final value in values) {
      if ((value as Enum).name == raw) return value;
    }
    return null;
  }
}

class StringArgCodec extends ArgCodec<String> {
  const StringArgCodec();

  @override
  String serialize(String value) => value;

  @override
  String? parse(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    return raw;
  }
}

class BoolArgCodec extends ArgCodec<bool> {
  const BoolArgCodec();

  @override
  String serialize(bool value) => value ? 'true' : 'false';

  @override
  bool? parse(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    switch (raw.toLowerCase()) {
      case 'true':
        return true;
      case 'false':
        return false;
      default:
        return null;
    }
  }
}

class IntArgCodec extends ArgCodec<int> {
  const IntArgCodec();

  @override
  String serialize(int value) => value.toString();

  @override
  int? parse(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    return int.tryParse(raw);
  }
}

class DoubleArgCodec extends ArgCodec<double> {
  const DoubleArgCodec();

  @override
  String serialize(double value) => value.toString();

  @override
  double? parse(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    return double.tryParse(raw);
  }
}

class EnumArgCodec<T extends Enum> extends ArgCodec<T> {
  const EnumArgCodec(this.values);

  final List<T> values;

  @override
  String serialize(T value) => value.name;

  @override
  T? parse(String? raw) {
    if (raw == null || raw.isEmpty) return null;
    for (final value in values) {
      if (value.name == raw) return value;
    }
    return null;
  }
}
