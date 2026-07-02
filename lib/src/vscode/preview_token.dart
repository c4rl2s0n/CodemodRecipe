import 'dart:convert';

import 'package:crypto/crypto.dart';

/// Canonical SHA-256 preview token for recipe requests + file snapshots.
String computePreviewToken(Map<String, Object?> payload) {
  final canonical = _canonicalize(payload);
  final bytes = utf8.encode(jsonEncode(canonical));
  return sha256.convert(bytes).toString();
}

Object? _canonicalize(Object? value) {
  if (value is Map) {
    final sortedKeys = value.keys.map((k) => k.toString()).toList()..sort();
    return {
      for (final key in sortedKeys)
        key: _canonicalize(value[key]),
    };
  }
  if (value is List) {
    return [for (final item in value) _canonicalize(item)];
  }
  return value;
}
