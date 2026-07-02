import 'dart:async';

import 'package:codemod_recipe/codemod_recipe_vscode.dart';
import 'package:codemod_recipe/src/mcp/codemod_mcp_server.dart';
import 'package:mcp_dart/mcp_dart.dart';

/// In-memory MCP client connected to a [createCodemodMcpServer] instance.
class CodemodMcpTestSession {
  CodemodMcpTestSession({
    required this.client,
    required this.server,
    required IOStreamTransport clientTransport,
    required IOStreamTransport serverTransport,
    required StreamController<List<int>> clientToServer,
    required StreamController<List<int>> serverToClient,
  }) : _clientTransport = clientTransport,
       _serverTransport = serverTransport,
       _clientToServer = clientToServer,
       _serverToClient = serverToClient;

  final McpClient client;
  final McpServer server;
  final IOStreamTransport _clientTransport;
  final IOStreamTransport _serverTransport;
  final StreamController<List<int>> _clientToServer;
  final StreamController<List<int>> _serverToClient;

  static Future<CodemodMcpTestSession> connect(CodemodHost host) async {
    final clientToServer = StreamController<List<int>>.broadcast();
    final serverToClient = StreamController<List<int>>.broadcast();

    final clientTransport = IOStreamTransport(
      stream: serverToClient.stream,
      sink: clientToServer.sink,
    );
    final serverTransport = IOStreamTransport(
      stream: clientToServer.stream,
      sink: serverToClient.sink,
    );

    final server = createCodemodMcpServer(host);
    await server.connect(serverTransport);

    final client = McpClient(
      const Implementation(name: 'codemod-mcp-test', version: '1.0'),
      options: const McpClientOptions(
        capabilities: ClientCapabilities(),
      ),
    );
    await client.connect(clientTransport);

    return CodemodMcpTestSession(
      client: client,
      server: server,
      clientTransport: clientTransport,
      serverTransport: serverTransport,
      clientToServer: clientToServer,
      serverToClient: serverToClient,
    );
  }

  Future<void> close() async {
    await client.close();
    await server.close();
    await _clientTransport.close();
    await _serverTransport.close();
    await _clientToServer.close();
    await _serverToClient.close();
  }
}
