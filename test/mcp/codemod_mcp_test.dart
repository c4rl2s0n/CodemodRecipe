import 'dart:convert';
import 'dart:io';

import 'package:codemod_recipe/codemod_recipe_vscode.dart';
import 'package:codemod_recipe/src/mcp/codemod_mcp_server.dart';
import 'package:mcp_dart/mcp_dart.dart';
import 'package:test/test.dart';

import 'mcp_test_harness.dart';

Map<String, Object?> _toolJson(CallToolResult result) {
  expect(result.content, isNotEmpty);
  final text = (result.content.first as TextContent).text;
  return jsonDecode(text) as Map<String, Object?>;
}

void main() {
  group('codemod-mcp integration (in-process)', () {
    late Directory workspace;
    late File settingsFile;
    late String settingsPath;
    late CodemodMcpTestSession session;

    setUp(() async {
      workspace = await Directory.systemTemp.createTemp('codemod_mcp_');
      settingsFile = File('${workspace.path}/lib/settings.dart')
        ..createSync(recursive: true);
      settingsFile.writeAsStringSync('''
class Settings {
  final int count = 0;
  final String name = 'x';

  void update() {
    print('hi');
  }
}
''');
      settingsPath = settingsFile.path;

      await Directory('${workspace.path}/.codemod/recipes').create(recursive: true);
      await File('test/fixtures/yaml_recipes/add_log_line.yaml').copy(
        '${workspace.path}/.codemod/recipes/add_log_line.yaml',
      );

      final host = CodemodHost.fromConfig(
        HostConfig(workspaceRoot: workspace.path, codemodRoot: '.codemod'),
      );
      session = await CodemodMcpTestSession.connect(host);
    });

    tearDown(() async {
      await session.close();
      workspace.deleteSync(recursive: true);
    });

    test('lists all registered MCP tools', () async {
      final tools = await session.client.listTools();
      expect(
        tools.tools.map((tool) => tool.name),
        containsAll(codemodMcpToolNames),
      );
    });

    test('list_recipes returns workspace recipes', () async {
      final result = await session.client.callTool(
        const CallToolRequest(name: 'list_recipes', arguments: {}),
      );

      final json = _toolJson(result);
      expect(json['ok'], isTrue);
      final recipes = json['recipes'] as List;
      expect(recipes.map((r) => (r as Map)['id']), contains('add_log_line'));
    });

    test('generate_ast_path returns navigate steps', () async {
      final offset = settingsFile.readAsStringSync().indexOf('count');
      final result = await session.client.callTool(
        CallToolRequest(
          name: 'generate_ast_path',
          arguments: {'path': settingsPath, 'offset': offset},
        ),
      );

      final json = _toolJson(result);
      expect(json['ok'], isTrue);
      final path = json['path'] as Map<String, Object?>;
      final navigate = path['navigate'] as List;
      expect(
        navigate.map((step) => (step as Map)['kind']),
        containsAll(['classDecl', 'field']),
      );
    });

    test('preview_recipe and apply_recipe inline remove round-trip', () async {
      final inlineRecipe = {
        'id': '__inline_remove_count',
        'steps': [
          {
            'edit': {
              'path': settingsPath,
              'steps': [
                {
                  'remove': {
                    'at': [
                      {'class': 'Settings'},
                      {'field': 'count'},
                    ],
                  },
                },
              ],
            },
          },
        ],
      };

      final preview = await session.client.callTool(
        CallToolRequest(
          name: 'preview_recipe',
          arguments: {'inlineRecipe': inlineRecipe},
        ),
      );
      final previewJson = _toolJson(preview);
      expect(previewJson['ok'], isTrue);
      expect(previewJson['previewToken'], isA<String>());
      expect((previewJson['files'] as List), isNotEmpty);

      final apply = await session.client.callTool(
        CallToolRequest(
          name: 'apply_recipe',
          arguments: {
            'inlineRecipe': inlineRecipe,
            'previewToken': previewJson['previewToken'],
          },
        ),
      );
      final applyJson = _toolJson(apply);
      expect(applyJson['ok'], isTrue);
      expect(applyJson['applied'], contains(settingsPath));

      final content = settingsFile.readAsStringSync();
      expect(content, isNot(contains('final int count')));
      expect(content, contains('final String name'));
    });

    test('apply_recipe schema requires previewToken', () async {
      await expectLater(
        session.client.callTool(
          const CallToolRequest(
            name: 'apply_recipe',
            arguments: {
              'inlineRecipe': {
                'id': '__inline_noop',
                'steps': [],
              },
            },
          ),
        ),
        throwsA(
          isA<McpError>().having(
            (error) => error.message,
            'message',
            contains('previewToken'),
          ),
        ),
      );
    });

    test('second preview is idempotent after apply', () async {
      final inlineRecipe = {
        'id': '__inline_remove_count',
        'steps': [
          {
            'edit': {
              'path': settingsPath,
              'steps': [
                {
                  'remove': {
                    'at': [
                      {'class': 'Settings'},
                      {'field': 'count'},
                    ],
                  },
                },
              ],
            },
          },
        ],
      };

      final firstPreview = _toolJson(
        await session.client.callTool(
          CallToolRequest(
            name: 'preview_recipe',
            arguments: {'inlineRecipe': inlineRecipe},
          ),
        ),
      );
      expect(firstPreview['ok'], isTrue);

      final apply = _toolJson(
        await session.client.callTool(
          CallToolRequest(
            name: 'apply_recipe',
            arguments: {
              'inlineRecipe': inlineRecipe,
              'previewToken': firstPreview['previewToken'],
            },
          ),
        ),
      );
      expect(apply['ok'], isTrue);

      final secondPreview = _toolJson(
        await session.client.callTool(
          CallToolRequest(
            name: 'preview_recipe',
            arguments: {'inlineRecipe': inlineRecipe},
          ),
        ),
      );
      expect(secondPreview['ok'], isTrue);
      expect(secondPreview['files'], isEmpty);
    });
  });

  group('codemod-mcp integration (subprocess)', () {
    late Directory workspace;
    late File settingsFile;
    late String settingsPath;
    late McpClient client;
    late StdioClientTransport transport;

    setUp(() async {
      workspace = await Directory.systemTemp.createTemp('codemod_mcp_proc_');
      settingsFile = File('${workspace.path}/lib/settings.dart')
        ..createSync(recursive: true);
      settingsFile.writeAsStringSync('''
class Settings {
  final int count = 0;
}
''');
      settingsPath = settingsFile.path;

      final packageRoot = Directory.current.path;
      transport = StdioClientTransport(
        StdioServerParameters(
          command: Platform.resolvedExecutable,
          args: [
            'run',
            'bin/codemod_mcp.dart',
            '--workspace-root',
            workspace.path,
            '--codemod-root',
            '.codemod',
          ],
          workingDirectory: packageRoot,
          environment: Platform.environment,
        ),
      );

      client = McpClient(
        const Implementation(name: 'codemod-mcp-subprocess-test', version: '1.0'),
        options: const McpClientOptions(
          capabilities: ClientCapabilities(),
        ),
      );
      await client.connect(transport);
    });

    tearDown(() async {
      await client.close();
      workspace.deleteSync(recursive: true);
    });

    test('stdio subprocess serves list_recipes and inline preview', () async {
      final list = _toolJson(
        await client.callTool(
          const CallToolRequest(name: 'list_recipes', arguments: {}),
        ),
      );
      expect(list['ok'], isTrue);

      final inlineRecipe = {
        'id': '__inline_remove_count',
        'steps': [
          {
            'edit': {
              'path': settingsPath,
              'steps': [
                {
                  'remove': {
                    'at': [
                      {'class': 'Settings'},
                      {'field': 'count'},
                    ],
                  },
                },
              ],
            },
          },
        ],
      };

      final preview = _toolJson(
        await client.callTool(
          CallToolRequest(
            name: 'preview_recipe',
            arguments: {'inlineRecipe': inlineRecipe},
          ),
        ),
      );
      expect(preview['ok'], isTrue);
      expect(preview['previewToken'], isA<String>());
    });
  });
}
