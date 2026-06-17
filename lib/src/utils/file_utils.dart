// Copyright (c) 2026, the Dart project authors. Please see the AUTHORS file
// for details. All rights reserved. Use of this source code is governed by
// a BSD-style license that can be found in the LICENSE file.

import 'dart:io';

/// Utility functions for file and directory operations.
///
/// This class centralizes common file system operations to improve
/// maintainability and follow the DRY principle.
class FileUtils {
  // File extensions
  static const yamlExtension = '.yaml';
  static const ymlExtension = '.yml';
  static const dartExtension = '.dart';
  static const defaultMapRoot = '.codemod/maps';

  /// Computes the relative path from [workspaceRoot] to [absolutePath].
  ///
  /// Both paths are normalized to use forward slashes for cross-platform consistency.
  static String relativePath(String workspaceRoot, String absolutePath) {
    final root = Directory(workspaceRoot).absolute.path.replaceAll('\\', '/');
    final file = File(absolutePath).absolute.path.replaceAll('\\', '/');
    
    if (file.startsWith('$root/')) {
      return file.substring(root.length + 1);
    }
    return file;
  }

  /// Checks if a file exists at the given [path].
  static Future<bool> fileExists(String path) async {
    return await File(path).exists();
  }

  /// Checks if a directory exists at the given [path].
  static Future<bool> directoryExists(String path) async {
    return await Directory(path).exists();
  }

  /// Reads the entire content of a file as a string.
  ///
  /// Returns null if the file does not exist.
  static Future<String?> readFileAsString(String path) async {
    final file = File(path);
    if (!await file.exists()) {
      return null;
    }
    return await file.readAsString();
  }

  /// Reads the entire content of a file as bytes.
  ///
  /// Returns null if the file does not exist.
  static Future<List<int>?> readFileAsBytes(String path) async {
    final file = File(path);
    if (!await file.exists()) {
      return null;
    }
    return await file.readAsBytes();
  }

  /// Writes [content] to a file at [path], creating parent directories if needed.
  ///
  /// Returns true if the file was written successfully, false otherwise.
  static Future<bool> writeFile(String path, String content) async {
    try {
      final file = File(path);
      final parent = file.parent;
      
      if (!await parent.exists()) {
        await parent.create(recursive: true);
      }
      
      await file.writeAsString(content);
      return true;
    } catch (_) {
      return false;
    }
  }

  /// Lists all files in a directory with the given [extensions].
  ///
  /// Returns a map of file paths to their content as strings.
  static Future<Map<String, String>> loadFilesWithExtensions(
    String directoryPath,
    List<String> extensions,
  ) async {
    final result = <String, String>{};
    
    final dir = Directory(directoryPath);
    if (!await dir.exists()) {
      return result;
    }

    try {
      await for (final entity in dir.list(recursive: false)) {
        if (entity is! File) continue;
        
        final filePath = entity.path;
        final extension = _getExtension(filePath).toLowerCase();
        
        if (extensions.contains(extension)) {
          try {
            final content = await entity.readAsString();
            result[filePath] = content;
          } catch (_) {
            // Skip files that cannot be read
            continue;
          }
        }
      }
    } catch (_) {
      // Ignore errors listing directory
    }
    
    return result;
  }

  /// Loads all YAML files from a directory.
  ///
  /// Returns a map of file paths to their YAML content.
  static Future<Map<String, String>> loadYamlFiles(String directoryPath) async {
    return await loadFilesWithExtensions(directoryPath, [yamlExtension, ymlExtension]);
  }

  /// Gets the absolute path of a file or directory.
  static String absolutePath(String path) {
    return path.isEmpty ? path : File(path).absolute.path;
  }

  /// Gets the directory name from a file path.
  static String dirname(String path) {
    return path.isEmpty ? '.' : path.split('/').take(path.split('/').length - 1).join('/');
  }

  /// Gets the base name (filename without extension) from a file path.
  static String basenameWithoutExtension(String path) {
    return path.isEmpty ? path : path.split('/').last.split('.').first;
  }

  /// Gets the file extension from a path.
  static String extension(String path) {
    return path.isEmpty ? '' : path.split('/').last.split('.').last;
  }

  /// Normalizes a path to use forward slashes and removes any trailing slashes.
  static String normalizePath(String path) {
    if (path.isEmpty) return path;
    
    // Replace all backslashes with forward slashes
    var normalized = path.replaceAll('\\', '/');
    
    // Remove trailing slash (but preserve root '/')
    if (normalized.length > 1 && normalized.endsWith('/')) {
      normalized = normalized.substring(0, normalized.length - 1);
    }
    
    return normalized;
  }

  /// Checks if a path is absolute.
  static bool isAbsolute(String path) {
    if (path.isEmpty) return false;
    return path.startsWith('/') || 
           (Platform.isWindows && path.length >= 2 && path[1] == ':');
  }

  /// Joins multiple path segments into a single path.
  static String joinPaths(List<String> segments) {
    final filtered = segments.where((s) => s.isNotEmpty).toList();
    return Uri(pathSegments: filtered).path;
  }

  /// Gets the current working directory path.
  static String getCurrentDirectory() {
    return Directory.current.absolute.path;
  }

  /// Checks if a file path ends with any of the given extensions.
  static bool hasExtension(String filePath, List<String> extensions) {
    if (filePath.isEmpty) return false;
    
    final fileName = filePath.split('/').last;
    final lastDot = fileName.lastIndexOf('.');
    if (lastDot < 0) return false;
    
    final fileExtension = fileName.substring(lastDot).toLowerCase();
    return extensions.any((ext) => fileExtension == ext.toLowerCase());
  }

  /// Extracts the extension from a file path.
  static String _getExtension(String filePath) {
    final fileName = filePath.split('/').last;
    final lastDot = fileName.lastIndexOf('.');
    if (lastDot < 0) return '';
    return fileName.substring(lastDot);
  }
}