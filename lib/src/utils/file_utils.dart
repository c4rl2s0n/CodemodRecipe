// Copyright (c) 2026, the Dart project authors. Please see the AUTHORS file
// for details. All rights reserved. Use of this source code is governed by
// a BSD-style license that can be found in the LICENSE file.

import 'dart:io';
import 'package:path/path.dart' as path;

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
    final root = Directory(workspaceRoot).absolute.path;
    final file = File(absolutePath).absolute.path;
    
    return path.relative(file, from: root);
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
        final extension = path.extension(filePath).toLowerCase();
        
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
  static String dirname(String filePath) {
    return path.dirname(filePath);
  }

  /// Gets the base name (filename without extension) from a file path.
  static String basenameWithoutExtension(String filePath) {
    return path.basenameWithoutExtension(filePath);
  }

  /// Gets the file extension from a path.
  static String extension(String filePath) {
    return path.extension(filePath);
  }

  /// Normalizes a path to use forward slashes and removes any trailing slashes.
  static String normalizePath(String inputPath) {
    return path.normalize(inputPath);
  }

  /// Checks if a path is absolute.
  static bool isAbsolute(String path) {
    if (path.isEmpty) return false;
    return path.startsWith('/') || 
           (Platform.isWindows && path.length >= 2 && path[1] == ':');
  }

  /// Joins multiple path segments into a single path.
  static String joinPaths(List<String> segments) {
    return path.joinAll(segments.where((s) => s.isNotEmpty).toList());
  }

  /// Gets the current working directory path.
  static String getCurrentDirectory() {
    return Directory.current.absolute.path;
  }

  /// Checks if a file path ends with any of the given extensions.
  static bool hasExtension(String filePath, List<String> extensions) {
    if (filePath.isEmpty) return false;
    
    final fileExtension = path.extension(filePath).toLowerCase();
    return extensions.any((ext) => fileExtension == ext.toLowerCase());
  }
}