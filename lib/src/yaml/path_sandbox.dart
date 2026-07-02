import 'dart:io';

import '../core/errors.dart';
import 'diagnostics.dart';
import 'host_config.dart';

/// Validates that [relativePath] resolves inside [config.workspaceRoot].
class PathSandbox {
  /// Creates a sandbox for [config].
  const PathSandbox(this.config);

  /// Host configuration providing workspace root.
  final HostConfig config;

  /// Resolves [relativePath] under the workspace or throws.
  String resolveWorkspaceRelative(String relativePath) {
    final normalized = _normalize(relativePath);
    if (normalized.startsWith('/')) {
      throw PathSandboxException(
        'Absolute paths are not allowed: $relativePath',
        code: 'E_PATH_TRAVERSAL',
      );
    }

    final resolved = _canonical(
      '${config.workspaceRoot.replaceAll('\\', '/')}/$normalized',
    );

    final root = _canonical(config.workspaceRoot.replaceAll('\\', '/'));
    if (!resolved.startsWith(root)) {
      throw PathSandboxException(
        'Path escapes workspace: $relativePath',
        code: 'E_PATH_TRAVERSAL',
      );
    }

    return resolved;
  }

  /// Resolves a template or script path under [config.codemodRoot].
  String resolveTemplateRelative(String relativePath) {
    final combined = '${config.codemodRoot}/$relativePath';
    return resolveWorkspaceRelative(combined);
  }

  /// Resolves a path relative to the codemod root directory.
  String resolveCodemodRelative(String relativePath) {
    final combined = '${config.codemodRoot}/$relativePath';
    return resolveWorkspaceRelative(combined);
  }

  static String _normalize(String path) {
    final segments = path
        .replaceAll('\\', '/')
        .split('/')
        .where((segment) => segment.isNotEmpty && segment != '.')
        .toList();

    if (segments.contains('..')) {
      throw PathSandboxException(
        'Path must not contain "..": $path',
        code: 'E_PATH_TRAVERSAL',
      );
    }

    return segments.join('/');
  }

  static String _canonical(String path) {
    return File(path).absolute.path.replaceAll('\\', '/');
  }
}

/// Converts a [PathSandboxException] to a [RecipeDiagnostic].
RecipeDiagnostic diagnosticFromSandbox(
  PathSandboxException error,
  String file,
) {
  return RecipeDiagnostics.pathSandbox(
    error.message,
    file,
    code: error.code,
  );
}
