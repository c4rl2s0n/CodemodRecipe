import 'dart:convert';

import 'package:codemod_recipe/codemod_recipe_vscode.dart';
import 'package:mcp_dart/mcp_dart.dart';

/// Creates an [McpServer] wired to [host] with all codemod MCP tools.
McpServer createCodemodMcpServer(CodemodHost host) {
  final server = McpServer(
    const Implementation(name: 'codemod-mcp', version: '0.1.0'),
    options: const McpServerOptions(
      capabilities: ServerCapabilities(
        tools: ServerCapabilitiesTools(),
      ),
    ),
  );

  Future<String> runHost(Map<String, Object?> request) async {
    final response = await host.dispatch(request);
    return jsonEncode(response);
  }

  server.registerTool(
    'list_recipes',
    description:
        'List codemod recipes registered in the workspace (.codemod/recipes plus '
        'Dart-registered recipes). Returns JSON: { ok, recipes[], diagnostics[] }. '
        'Use describe_recipe for full arg metadata.',
    inputSchema: JsonSchema.object(
      description: 'No arguments.',
    ),
    callback: (args, extra) async {
      final text = await runHost({'command': 'list'});
      return CallToolResult.fromContent([TextContent(text: text)]);
    },
  );

  server.registerTool(
    'describe_recipe',
    description:
        'Describe one registered recipe: args, inputKind, options, template previews. '
        'Returns JSON: { ok, recipe } or { ok: false, error }.',
    inputSchema: JsonSchema.object(
      description: 'Recipe id from list_recipes.',
      properties: {
        'recipe': JsonSchema.string(
          description: 'Registered recipe id (e.g. add_log_line).',
        ),
      },
      required: ['recipe'],
    ),
    callback: (args, extra) async {
      final text = await runHost({
        'command': 'describe',
        'recipe': args['recipe'],
      });
      return CallToolResult.fromContent([TextContent(text: text)]);
    },
  );

  server.registerTool(
    'validate_recipes',
    description:
        'Reload and validate all YAML recipes and maps under .codemod/. '
        'Returns JSON: { ok, diagnostics[] }. ok is false when any error-severity diagnostic exists.',
    inputSchema: JsonSchema.object(
      description: 'No arguments.',
    ),
    callback: (args, extra) async {
      final text = await runHost({'command': 'validate'});
      return CallToolResult.fromContent([TextContent(text: text)]);
    },
  );

  server.registerTool(
    'preview_recipe',
    description:
        'Dry-run a recipe without writing files. Returns previewToken (required for '
        'apply_recipe), per-file patch snippets, and ok/files/recipe. '
        'Pass either recipe (registered id) or inlineRecipe (YAML-shaped object). '
        'Empty files[] means no changes (idempotent). See docs/codemod-mcp.md.',
    inputSchema: JsonSchema.object(
      description:
          'Provide recipe OR inlineRecipe. Args are string key/value pairs.',
      properties: {
        'recipe': JsonSchema.string(
          description: 'Registered recipe id from list_recipes.',
        ),
        'inlineRecipe': JsonSchema.object(
          description:
              'Inline recipe body: { id, steps: [{ edit: { path, steps: [...] } }] }. '
              'edit.path should be an absolute file path. Supports insert, remove, replace steps.',
        ),
        'args': JsonSchema.object(
          description:
              'Recipe argument values (e.g. file, className, methodName for registered recipes).',
        ),
        'snippetLines': JsonSchema.number(
          description: 'Lines of context around each patch preview (1–20, default 5).',
        ),
      },
    ),
    callback: (args, extra) async {
      final request = <String, Object?>{'command': 'preview'};
      if (args['recipe'] != null) request['recipe'] = args['recipe'];
      if (args['inlineRecipe'] != null) {
        request['inlineRecipe'] = args['inlineRecipe'];
      }
      if (args['args'] != null) request['args'] = args['args'];
      if (args['snippetLines'] != null) {
        request['snippetLines'] = args['snippetLines'];
      }
      final text = await runHost(request);
      return CallToolResult.fromContent([TextContent(text: text)]);
    },
  );

  server.registerTool(
    'apply_recipe',
    description:
        'Apply a previewed recipe atomically (rollback on failure). '
        'REQUIRES previewToken from preview_recipe with the same recipe/inlineRecipe and args. '
        'Returns JSON: { ok, applied[] } or stale/missing token errors.',
    inputSchema: JsonSchema.object(
      description: 'Must include previewToken from the matching preview_recipe call.',
      properties: {
        'previewToken': JsonSchema.string(
          description: 'SHA-256 token from preview_recipe. Re-preview if files changed.',
        ),
        'recipe': JsonSchema.string(
          description: 'Same registered recipe id used in preview.',
        ),
        'inlineRecipe': JsonSchema.object(
          description: 'Same inline recipe object used in preview.',
        ),
        'args': JsonSchema.object(
          description: 'Same args object used in preview.',
        ),
        'selection': JsonSchema.object(
          description:
              'Optional patch subset: { files: { "<path>": { include, patches: [indices] } } }.',
        ),
      },
      required: ['previewToken'],
    ),
    callback: (args, extra) async {
      final request = <String, Object?>{'command': 'apply'};
      if (args['recipe'] != null) request['recipe'] = args['recipe'];
      if (args['inlineRecipe'] != null) {
        request['inlineRecipe'] = args['inlineRecipe'];
      }
      if (args['args'] != null) request['args'] = args['args'];
      request['previewToken'] = args['previewToken'];
      if (args['selection'] != null) request['selection'] = args['selection'];
      final text = await runHost(request);
      return CallToolResult.fromContent([TextContent(text: text)]);
    },
  );

  server.registerTool(
    'generate_ast_path',
    description:
        'Convert a Dart file path and byte offset into AST navigate steps for inline '
        'recipe at: blocks. Returns JSON: { ok, path: { navigate[], anchor, offset } }. '
        'For remove/replace, use navigate only (omit anchor). Pair with codebase-memory '
        'get_code_snippet offsets.',
    inputSchema: JsonSchema.object(
      description: 'Absolute path recommended for path.',
      properties: {
        'path': JsonSchema.string(
          description: 'Dart source file path.',
        ),
        'offset': JsonSchema.number(
          description: '0-based byte offset into the file (from editor or get_code_snippet).',
        ),
      },
      required: ['path', 'offset'],
    ),
    callback: (args, extra) async {
      final text = await runHost({
        'command': 'generateAstPath',
        'path': args['path'],
        'offset': args['offset'],
      });
      return CallToolResult.fromContent([TextContent(text: text)]);
    },
  );

  return server;
}

/// All tool names registered by [createCodemodMcpServer].
const codemodMcpToolNames = [
  'list_recipes',
  'describe_recipe',
  'validate_recipes',
  'preview_recipe',
  'apply_recipe',
  'generate_ast_path',
];
