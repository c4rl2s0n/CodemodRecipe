import * as path from 'path';
import * as vscode from 'vscode';
import { COMMANDS, VIEWS } from './constants';
import { ExtensionConfig } from './config/extensionConfig';
import { DiffContentProvider } from './diff/diffContentProvider';
import { DartBridge } from './host/dartBridge';
import { prefillArgs, resolveEditorContext } from './recipes/recipeContext';
import { RecipeRepository } from './recipes/recipeRepository';
import type { RecipeSchema, AstPathResult } from '../shared';
import { RecipeRunnerViewProvider } from './views/recipeRunnerViewProvider';

export function activate(context: vscode.ExtensionContext): void {
  const workspaceRoot = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (!workspaceRoot) {
    return;
  }

  const config = new ExtensionConfig();
  const bridge = new DartBridge(workspaceRoot, config, context.extensionUri);
  const diffProvider = new DiffContentProvider();
  const repository = new RecipeRepository(bridge);
  const runner = new RecipeRunnerViewProvider(
    bridge,
    config,
    diffProvider,
    workspaceRoot,
    context.extensionUri
  );

  let recipeReloadTimer: NodeJS.Timeout | undefined;
  let codemodWatcher: vscode.FileSystemWatcher | undefined;

  const syncRunnerFromRepository = async (): Promise<void> => {
    await runner.refreshRecipes(
      repository.getRecipes(),
      repository.getLastError(),
      repository.getDiagnostics()
    );
  };

  const reloadRecipesFromHost = async (showError = false): Promise<void> => {
    runner.setRecipesRefreshing(true);
    try {
      await bridge.ensureHost();
      try {
        await repository.reload();
      } catch {
        bridge.dispose();
        await bridge.ensureHost();
        await repository.refresh();
      }
      await syncRunnerFromRepository();
      if (showError && repository.getLastError()) {
        vscode.window.showWarningMessage(
          `Codemod Recipe: ${repository.getLastError()}`
        );
      }
    } finally {
      runner.setRecipesRefreshing(false);
    }
  };

  const restartHostAndRefresh = async (showError = false): Promise<void> => {
    runner.setRecipesRefreshing(true);
    try {
      bridge.dispose();
      await bridge.ensureHost();
      await repository.refresh();
      await syncRunnerFromRepository();
      if (showError && repository.getLastError()) {
        vscode.window.showWarningMessage(
          `Codemod Recipe: ${repository.getLastError()}`
        );
      }
    } finally {
      runner.setRecipesRefreshing(false);
    }
  };

  const scheduleRecipeReload = (): void => {
    if (recipeReloadTimer) {
      clearTimeout(recipeReloadTimer);
    }
    recipeReloadTimer = setTimeout(() => {
      void reloadRecipesFromHost();
    }, 300);
  };

  const disposeCodemodWatcher = (): void => {
    codemodWatcher?.dispose();
    codemodWatcher = undefined;
  };

  const createCodemodWatcher = (): void => {
    disposeCodemodWatcher();
    const codemodRootDir = path.join(workspaceRoot, config.codemodRoot);
    
    // Watch for YAML files (recipes and maps) and .template files
    codemodWatcher = vscode.workspace.createFileSystemWatcher(
      new vscode.RelativePattern(
        codemodRootDir,
        '**/*.{yaml,yml,template}'
      )
    );
    codemodWatcher.onDidChange(scheduleRecipeReload);
    codemodWatcher.onDidCreate(scheduleRecipeReload);
    codemodWatcher.onDidDelete(scheduleRecipeReload);
  };

  const bootstrap = async (showError = false): Promise<void> => {
    runner.setBootstrap({ inFlight: true, phase: 'startingHost' });
    try {
      createCodemodWatcher();
      runner.setBootstrap({ inFlight: true, phase: 'loadingRecipes' });
      await restartHostAndRefresh(showError);
      runner.setBootstrap({ inFlight: false, phase: 'ready' });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      runner.setBootstrap({ inFlight: false, phase: 'error', error: message });
    }
  };

  context.subscriptions.push(
    { dispose: () => bridge.dispose() },
    { dispose: () => disposeCodemodWatcher() },
    vscode.workspace.onDidChangeConfiguration((event) => {
      if (
        event.affectsConfiguration('codemodRecipe.codemodRoot') ||
        event.affectsConfiguration('codemodRecipe.emptyConstructorStyle') ||
        event.affectsConfiguration('codemodRecipe.dartPath')
      ) {
        bridge.dispose();
        void bootstrap(true);
      }
    }),
    vscode.workspace.registerTextDocumentContentProvider(
      DiffContentProvider.scheme,
      diffProvider
    ),
    vscode.window.registerWebviewViewProvider(
      VIEWS.runner,
      runner,
      { webviewOptions: { retainContextWhenHidden: true } }
    ),
    vscode.commands.registerCommand(COMMANDS.refresh, () =>
      reloadRecipesFromHost(true)
    ),
    vscode.commands.registerCommand(COMMANDS.bootstrap, () => bootstrap(true)),
    vscode.commands.registerCommand(
      COMMANDS.runRecipe,
      async (recipe?: RecipeSchema) => {
        if (!recipe) {
          const picked = await vscode.window.showQuickPick(
            repository.getRecipes().map((item) => ({
              label: item.name,
              description: item.description,
              recipe: item,
            })),
            { placeHolder: 'Select a codemod recipe' }
          );
          recipe = picked?.recipe;
        }
        if (recipe) {
          runner.run(recipe);
        }
      }
    ),
    vscode.commands.registerCommand(
      COMMANDS.runFromCursorContext,
      async () => {
        const recipes = repository.getRecipes();
        const editorContext = resolveEditorContext(workspaceRoot);
        const candidates = recipes
          .map((recipe) => ({
            recipe,
            args: prefillArgs(recipe, editorContext.values),
          }))
          .filter((candidate) => Object.keys(candidate.args).length > 0);

        if (candidates.length === 0) {
          vscode.window.showInformationMessage(
            'No recipes declare arguments that match the current editor context.'
          );
          return;
        }

        const picked = await vscode.window.showQuickPick(
          candidates.map((candidate) => ({
            label: candidate.recipe.name,
            description: candidate.recipe.description,
            detail: Object.entries(candidate.args)
              .map(([key, value]) => `${key}: ${value}`)
              .join(', '),
            candidate,
          })),
          { placeHolder: 'Run recipe using values from the current cursor context' }
        );

        if (picked) {
          runner.run(picked.candidate.recipe, picked.candidate.args);
        }
      }
    ),
    vscode.commands.registerCommand(
      COMMANDS.configureCodemodRoot,
      async () => {
        const value = await vscode.window.showInputBox({
          prompt:
            'Path (relative to workspace) of the codemod root directory',
          placeHolder: '.codemod',
        });
        if (value !== undefined) {
          await config.updateCodemodRoot(value);
          bridge.dispose();
          await bootstrap(true);
        }
      }
    ),
    vscode.commands.registerCommand(
      COMMANDS.generateAstPath,
      async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
          vscode.window.showWarningMessage('No active editor');
          return;
        }
        
        const offset = editor.document.offsetAt(editor.selection.active);
        const filePath = editor.document.uri.fsPath;
        
        if (!filePath.endsWith('.dart')) {
          vscode.window.showWarningMessage('AST path generation only works for Dart files');
          return;
        }
        
        try {
          const result: AstPathResult = await bridge.generateAstPath(filePath, offset);
          if (!result.ok) {
            throw new Error(result.error || 'Unknown error');
          }
          if (!result.path) {
            throw new Error('AST path result missing path data');
          }
          const astPath = result.path;
          
          // Show quick pick to choose output format
          const formatChoice = await vscode.window.showQuickPick([
            { label: '📋 Compact Localization', description: 'Show compact format (e.g., "class:Name > method:name @ anchor")' },
            { label: '📄 Full YAML Recipe', description: 'Generate complete YAML recipe' },
            { label: '📝 Copy to Clipboard', description: 'Copy compact format to clipboard' }
          ], {
            placeHolder: 'Choose output format for AST path'
          });
          
          if (!formatChoice) return;
          
          // Show anchor preview visualization
          _showAnchorPreview(editor, offset, astPath.anchor);
          
          if (formatChoice.label.includes('Copy to Clipboard')) {
            const compact = _generateCompactLocalization(astPath);
            await vscode.env.clipboard.writeText(compact);
            vscode.window.showInformationMessage('📋 Compact localization copied to clipboard!');
          } else if (formatChoice.label.includes('Full YAML Recipe')) {
            const yaml = _generateYamlFromAstPath(astPath, filePath);
            const doc = await vscode.workspace.openTextDocument({
              content: yaml,
              language: 'yaml'
            });
            await vscode.window.showTextDocument(doc);
          } else {
            // Show compact format in a nice information message
            const compact = _generateCompactLocalization(astPath);
            
            // Show as information message with copy option
            const action = await vscode.window.showInformationMessage(
              `🎯 AST Path: ${compact}`,
              'Copy', 'Open Full Recipe'
            );
            
            if (action === 'Copy') {
              await vscode.env.clipboard.writeText(compact);
              vscode.window.showInformationMessage('📋 Copied to clipboard!');
            } else if (action === 'Open Full Recipe') {
              const yaml = _generateYamlFromAstPath(astPath, filePath);
              const doc = await vscode.workspace.openTextDocument({
                content: yaml,
                language: 'yaml'
              });
              await vscode.window.showTextDocument(doc);
            }
          }
          
        } catch (error) {
          vscode.window.showErrorMessage(`Failed to generate AST path: ${error}`);
        }
      }
    )
  );

  void bootstrap();
}

export function deactivate(): void {
  // No-op: child processes are short-lived and exit on their own.
}

function _generateYamlFromAstPath(astPath: any, filePath: string): string {
  const stepsYaml = astPath.navigate.map((step: any) => {
    const parts = [];
    if (step.kind) {
      parts.push(`${step.kind}: "${step.name}"`);
    } else {
      parts.push(`inferred: "${step.name}"`);
    }
    if (step.match) {
      parts.push(`# match: "${step.match}"`);
    }
    return `              - ${parts.join(' ')}`;
  }).join('\n');

  return `dslVersion: 1
id: generated_${_sanitizeFileName(path.basename(filePath))}_${Date.now()}
name: "Generated Recipe from ${path.basename(filePath)}"
description: "Recipe generated from AST path at offset ${astPath.offset}"

args:
  - name: file
    required: true
    inputKind: file
    help: "The file to modify"

steps:
  - edit:
      path: "{{file}}"
      steps:
        - insert:
            at:
${stepsYaml}
            anchor: ${astPath.anchor}
            text: "// TODO: Add your code here"

postExecution:
  - run: dart format .`;
}

function _sanitizeFileName(filename: string): string {
  return filename.replace(/[^a-zA-Z0-9_]/g, '_');
}

// Decoration for showing anchor preview
let anchorPreviewDecoration: vscode.TextEditorDecorationType;

function _showAnchorPreview(editor: vscode.TextEditor, offset: number, anchorType: string) {
  // Create decoration type if it doesn't exist
  if (!anchorPreviewDecoration) {
    anchorPreviewDecoration = vscode.window.createTextEditorDecorationType({
      backgroundColor: 'rgba(255, 200, 0, 0.2)',
      border: '1px solid rgba(255, 165, 0, 0.5)',
      borderRadius: '2px',
      isWholeLine: true,
      overviewRulerLane: vscode.OverviewRulerLane.Right,
      overviewRulerColor: 'rgba(255, 165, 0, 0.5)'
    });
  }
  
  // Calculate the position to highlight based on anchor type
  const document = editor.document;
  const position = document.positionAt(offset);
  
  // Determine what to highlight based on anchor type
  let range: vscode.Range;
  
  if (anchorType.includes('stmtLast') || anchorType.includes('bodyEnd')) {
    // Highlight the line where new code would be inserted after
    const line = document.lineAt(position.line);
    range = new vscode.Range(line.range.end, line.range.end);
  } else if (anchorType.includes('argLast') || anchorType.includes('paramLast')) {
    // Highlight the argument list
    const line = document.lineAt(position.line);
    range = new vscode.Range(line.range.start, line.range.end);
  } else if (anchorType.includes('memberLast')) {
    // Highlight the last member
    range = document.lineAt(position.line).range;
  } else {
    // Default: highlight the current line
    range = document.lineAt(position.line).range;
  }
  
  // Apply the decoration
  editor.setDecorations(anchorPreviewDecoration, [range]);
  
  // Show a temporary message
  setTimeout(() => {
    editor.setDecorations(anchorPreviewDecoration, []);
  }, 5000);
}

function _generateCompactLocalization(astPath: any): string {
  const steps = astPath.navigate.map((step: any) => {
    if (step.kind) {
      return `${step.kind}:${step.name}`;
    } else {
      return `inferred:${step.name}`;
    }
  }).join(' > ');
  
  return `${steps} @ ${astPath.anchor}`;
}
