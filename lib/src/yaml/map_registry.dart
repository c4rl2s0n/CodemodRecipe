import 'dart:io';

import 'package:yaml/yaml.dart';

import 'diagnostics.dart';

/// A named string-to-string mapping loaded from YAML.
class YamlStringMap {
  const YamlStringMap({required this.id, required this.entries});

  final String id;
  final Map<String, String> entries;
}

/// Result of loading YAML maps.
class YamlMapLoadResult {
  const YamlMapLoadResult({required this.mapsById, required this.diagnostics});

  final Map<String, Map<String, String>> mapsById;
  final List<RecipeDiagnostic> diagnostics;
}

/// Loads reusable YAML string maps from a directory.
class YamlMapRegistry {
  static YamlMapLoadResult load({
    required String workspaceRoot,
    required String mapsDirectoryPath,
  }) {
    final diagnostics = <RecipeDiagnostic>[];
    final idSources = <String, List<DiagnosticSource>>{};
    final mapsById = <String, Map<String, String>>{};

    final directory = Directory(mapsDirectoryPath);
    if (!directory.existsSync()) {
      return YamlMapLoadResult(mapsById: mapsById, diagnostics: diagnostics);
    }

    for (final file
        in directory
            .listSync(recursive: true)
            .whereType<File>()
            .where((file) => _isYaml(file.path))) {
      final relativePath = _relativePath(workspaceRoot, file.path);
      try {
        final doc = loadYaml(file.readAsStringSync());
        if (doc is! YamlMap) {
          diagnostics.add(
            RecipeDiagnostics.mapSchemaError(
              'Map file root must be a map',
              relativePath,
            ),
          );
          continue;
        }

        final id = doc['id']?.toString();
        if (id == null || id.isEmpty) {
          diagnostics.add(
            RecipeDiagnostics.mapSchemaError(
              'Map file missing required "id"',
              relativePath,
            ),
          );
          continue;
        }

        final entriesNode = doc['entries'];
        if (entriesNode is! YamlMap) {
          diagnostics.add(
            RecipeDiagnostics.mapSchemaError(
              'Map "$id" missing required "entries" map',
              relativePath,
            ),
          );
          continue;
        }

        final entries = <String, String>{};
        for (final entry in entriesNode.entries) {
          final key = entry.key.toString();
          final value = entry.value?.toString() ?? '';
          entries[key] = value;
        }

        mapsById[id] = entries;
        idSources
            .putIfAbsent(id, () => [])
            .add(DiagnosticSource(file: relativePath));
      } catch (error) {
        diagnostics.add(
          RecipeDiagnostics.mapParseError('$error', relativePath),
        );
      }
    }

    final rejectedIds = <String>{};
    for (final entry in idSources.entries) {
      if (entry.value.length < 2) continue;
      rejectedIds.add(entry.key);
      diagnostics.add(RecipeDiagnostics.duplicateMapId(entry.key, entry.value));
    }

    for (final id in rejectedIds) {
      mapsById.remove(id);
    }

    return YamlMapLoadResult(mapsById: mapsById, diagnostics: diagnostics);
  }

  static bool _isYaml(String path) =>
      path.endsWith('.yaml') || path.endsWith('.yml');

  static String _relativePath(String workspaceRoot, String absolutePath) {
    final root = Directory(workspaceRoot).absolute.path.replaceAll('\\', '/');
    final file = File(absolutePath).absolute.path.replaceAll('\\', '/');
    if (file.startsWith('$root/')) {
      return file.substring(root.length + 1);
    }
    return file;
  }
}
