import 'dart:io';

import 'package:codemod_recipe/codemod_recipe_vscode.dart';
import 'package:codemod_recipe/src/mcp/codemod_mcp_server.dart';
import 'package:mcp_dart/mcp_dart.dart';

/// MCP server exposing [CodemodHost] commands as agent tools.
///
/// **Documentation:** [docs/codemod-mcp.md](../docs/codemod-mcp.md)
/// **Agent playbook:** `.cursor/skills/codemod-mcp/reference.md`
///
/// Usage:
/// ```bash
/// dart run bin/codemod_mcp.dart --workspace-root . --codemod-root .codemod
/// ```
///
/// All tools return a JSON string. Parse it and check `ok` before using fields.
/// Preview before apply; [apply_recipe] requires `previewToken` from [preview_recipe].
Future<void> main(List<String> arguments) async {
  final parser = HostConfig.buildArgParser();
  final results = parser.parse(arguments);
  final config = HostConfig.fromArgResults(results);
  final host = CodemodHost.fromConfig(config);
  final server = createCodemodMcpServer(host);

  stderr.writeln('codemod-mcp ready (workspace: ${config.workspaceRoot})');
  await server.connect(StdioServerTransport());
}
