import * as path from 'path';
import * as vscode from 'vscode';
import { COMMANDS, VIEWS } from '../constants';
import type { ExtensionConfig } from '../config/extensionConfig';
import type { HostBridge } from '../host/hostBridge';
import type { DebugMatchDto } from '../../shared';
import { AstTreeItem, AstTreeProvider } from './astTreeProvider';
import { QueryToolsDecorations } from './decorations';
import {
  formatYamlOp,
  QueryToolsPanelProvider,
} from './queryToolsPanel';
import {
  findEditPathNearLine,
  extractQueryCaptureAnchor,
} from './recipeYamlExtract';

export class QueryToolsController implements vscode.Disposable {
  private readonly astProvider = new AstTreeProvider();
  private readonly panel = new QueryToolsPanelProvider();
  private readonly decorations = new QueryToolsDecorations();
  private readonly disposables: vscode.Disposable[] = [];
  private matches: DebugMatchDto[] = [];
  private focusIndex = 0;
  private boundUri: vscode.Uri | undefined;
  private astTreeView?: vscode.TreeView<AstTreeItem>;
  private revealTimer: NodeJS.Timeout | undefined;
  /** Document version when AST was last dumped for the tree view. */
  private lastDumpVersion = -1;
  private astStale = false;

  constructor(
    private readonly bridge: HostBridge,
    private readonly config: ExtensionConfig
  ) {}

  register(context: vscode.ExtensionContext): void {
    this.astTreeView = vscode.window.createTreeView(VIEWS.queryAst, {
      treeDataProvider: this.astProvider,
      showCollapseAll: true,
    });

    this.disposables.push(
      this.astTreeView,
      vscode.window.registerWebviewViewProvider(
        QueryToolsPanelProvider.viewType,
        this.panel,
        { webviewOptions: { retainContextWhenHidden: true } }
      ),
      this.panel.onRun(() => {
        void this.runQuery();
      }),
      this.panel.onStateChange((s) => {
        if (typeof s.focusIndex === 'number') {
          this.focusIndex = s.focusIndex;
        }
        if (this.matches.length) {
          this.paintMatches(s.capture, s.anchor);
        }
      }),
      this.panel.onCopy((kind) => {
        void this.copy(kind);
      }),
      vscode.commands.registerCommand(
        'codemodRecipe.queryTools.selectAstNode',
        (item: AstTreeItem) => {
          void this.onSelectAstNode(item);
        }
      ),
      vscode.commands.registerCommand(COMMANDS.queryToolsRun, () =>
        void this.runQuery()
      ),
      vscode.commands.registerCommand(
        COMMANDS.queryToolsGenerateFromCursor,
        () => void this.generateFromCursor()
      ),
      vscode.commands.registerCommand(COMMANDS.queryToolsRevealAst, () =>
        void this.revealAstFromCursor()
      ),
      vscode.commands.registerCommand(COMMANDS.queryToolsNextMatch, () =>
        this.shiftMatch(1)
      ),
      vscode.commands.registerCommand(COMMANDS.queryToolsPrevMatch, () =>
        this.shiftMatch(-1)
      ),
      vscode.commands.registerCommand(COMMANDS.queryToolsCopy, () =>
        void this.copy('query')
      ),
      vscode.commands.registerCommand(COMMANDS.queryToolsCopyYamlInsert, () =>
        void this.copy('insert')
      ),
      vscode.commands.registerCommand(COMMANDS.queryToolsCopyYamlReplace, () =>
        void this.copy('replace')
      ),
      vscode.commands.registerCommand(COMMANDS.queryToolsCopyYamlRemove, () =>
        void this.copy('remove')
      ),
      vscode.commands.registerCommand(
        COMMANDS.queryToolsOpenFromRecipe,
        (doc?: vscode.TextDocument, line?: number) =>
          void this.openFromRecipe(doc, line)
      ),
      vscode.commands.registerCommand(
        COMMANDS.queryToolsGoToEditPath,
        (doc?: vscode.TextDocument, line?: number) =>
          void this.goToEditPath(doc, line)
      ),
      this.astTreeView.onDidChangeVisibility((e) => {
        if (e.visible) {
          void this.refreshAstIfVisible(true);
        }
      }),
      vscode.window.onDidChangeActiveTextEditor(() => {
        void this.refreshAstIfVisible(false);
      }),
      vscode.workspace.onDidSaveTextDocument((doc) => {
        if (
          this.isAstViewVisible() &&
          doc.uri.toString() === this.boundUri?.toString()
        ) {
          void this.refreshAst(true);
        }
      }),
      vscode.workspace.onDidChangeTextDocument((e) => {
        if (e.document.uri.toString() !== this.boundUri?.toString()) {
          return;
        }
        if (e.document.version !== this.lastDumpVersion) {
          this.astStale = true;
          this.updateStaleStatus();
        }
      }),
      vscode.window.onDidChangeTextEditorSelection((e) => {
        if (e.textEditor.document.uri.toString() !== this.boundUri?.toString()) {
          return;
        }
        this.scheduleRevealAtCursor();
      })
    );
    context.subscriptions.push(this);
  }

  private isAstViewVisible(): boolean {
    return this.astTreeView?.visible ?? false;
  }

  private scheduleRevealAtCursor(): void {
    if (!this.isAstViewVisible() || this.astStale) {
      return;
    }
    if (this.revealTimer) {
      clearTimeout(this.revealTimer);
    }
    this.revealTimer = setTimeout(() => {
      void this.revealAtCursor(false);
    }, 150);
  }

  private async refreshAstIfVisible(force: boolean): Promise<void> {
    if (!this.isAstViewVisible()) {
      return;
    }
    await this.refreshAst(force);
  }

  private updateStaleStatus(): void {
    if (!this.astStale) {
      return;
    }
    const base = this.astProvider.getHasError()
      ? 'Parse has ERROR nodes — query matching may fail.'
      : '';
    const stale = 'Stale (unsaved) — save to refresh AST tree.';
    this.panel.setStatus(base ? `${stale}\n${base}` : stale);
    if (this.astTreeView) {
      this.astTreeView.description = 'Stale (unsaved)';
    }
  }

  private clearStaleStatus(): void {
    this.astStale = false;
    if (this.astTreeView) {
      this.astTreeView.description = undefined;
    }
  }

  private activeEditor(): vscode.TextEditor | undefined {
    const ed = vscode.window.activeTextEditor;
    if (!ed || ed.document.uri.scheme !== 'file') {
      return undefined;
    }
    if (this.boundUri) {
      const found = vscode.window.visibleTextEditors.find(
        (e) => e.document.uri.toString() === this.boundUri!.toString()
      );
      if (found) {
        return found;
      }
    }
    return ed;
  }

  private relativePath(uri: vscode.Uri): string {
    return path.relative(this.config.workspaceRoot, uri.fsPath).replace(/\\/g, '/');
  }

  private async refreshAst(force: boolean): Promise<void> {
    const ed = this.activeEditor();
    if (!ed) {
      this.astProvider.setRoot(undefined, false);
      return;
    }
    if (
      !force &&
      !this.astStale &&
      ed.document.version === this.lastDumpVersion &&
      this.astProvider.getRoot()
    ) {
      return;
    }
    this.boundUri = ed.document.uri;
    const rel = this.relativePath(ed.document.uri);
    try {
      const resp = await this.bridge.dumpAst({
        source: ed.document.getText(),
        path: rel,
        namedOnly: true,
      });
      if (!resp.ok || !resp.root) {
        this.panel.setStatus(resp.error ?? 'dumpAst failed');
        this.astProvider.setRoot(undefined, false);
        return;
      }
      this.astProvider.setRoot(resp.root, !!resp.hasError);
      this.lastDumpVersion = ed.document.version;
      this.clearStaleStatus();
      if (resp.hasError) {
        this.panel.setStatus('Parse has ERROR nodes — query matching may fail.');
      } else if (!this.panel.getState().query.trim()) {
        this.panel.setStatus('AST ready — select a node or Generate from cursor.');
      }
    } catch (e) {
      this.panel.setStatus(e instanceof Error ? e.message : String(e));
    }
  }

  private async revealAtCursor(focusAst: boolean): Promise<void> {
    const ed = this.activeEditor();
    if (!ed || !this.astProvider.getRoot()) {
      return;
    }
    const offset = ed.document.offsetAt(ed.selection.active);
    const pathIdx = this.astProvider.findPathForByte(offset);
    if (!pathIdx) {
      return;
    }
    const item = this.astProvider.getItemForPath(pathIdx);
    if (!item) {
      return;
    }
    this.decorations.showAstSelection(ed, item.node);
    try {
      await this.astTreeView?.reveal(item, {
        expand: true,
        select: true,
        focus: focusAst,
      });
    } catch {
      // reveal can fail if the item is not yet rendered
    }
  }

  private async onSelectAstNode(item: AstTreeItem): Promise<void> {
    const ed = this.activeEditor();
    if (!ed) {
      return;
    }
    this.decorations.showAstSelection(ed, item.node);
    const pinEq = this.panel.isPinEq();
    const rel = this.relativePath(ed.document.uri);
    try {
      const resp = await this.bridge.generateQuery({
        source: ed.document.getText(),
        path: rel,
        start: item.node.start.byte,
        end: item.node.end.byte,
        includeTextPredicates: pinEq,
        captureLeaf: 'target',
        maxDepth: 8,
      });
      if (!resp.ok || !resp.query) {
        this.panel.setStatus(resp.error ?? 'generateQuery failed');
        return;
      }
      this.panel.setQuery(resp.query, resp.captureSuggestion);
      this.panel.setStatus(`Generated query for ${item.node.kind}`);
      if (this.isAstViewVisible()) {
        await this.refreshAst(true);
      }
    } catch (e) {
      this.panel.setStatus(e instanceof Error ? e.message : String(e));
    }
  }

  private async generateFromCursor(): Promise<void> {
    const ed = this.activeEditor();
    if (!ed) {
      vscode.window.showWarningMessage('Query Tools: no active editor');
      return;
    }
    const offset = ed.document.offsetAt(ed.selection.active);
    const rel = this.relativePath(ed.document.uri);
    const pinEq = this.panel.isPinEq();
    try {
      const resp = await this.bridge.generateQuery({
        source: ed.document.getText(),
        path: rel,
        start: offset,
        end: offset,
        includeTextPredicates: pinEq,
        captureLeaf: 'target',
        maxDepth: 8,
      });
      if (!resp.ok || !resp.query) {
        this.panel.setStatus(resp.error ?? 'generateQuery failed');
        return;
      }
      this.panel.setQuery(resp.query, resp.captureSuggestion);
      this.panel.setStatus('Generated from cursor');
      if (this.isAstViewVisible()) {
        await this.refreshAst(true);
        await this.revealAtCursor(false);
      }
      await vscode.commands.executeCommand(`${VIEWS.queryEditor}.focus`);
    } catch (e) {
      this.panel.setStatus(e instanceof Error ? e.message : String(e));
    }
  }

  private async revealAstFromCursor(): Promise<void> {
    const ed = this.activeEditor();
    if (!ed) {
      return;
    }
    await this.refreshAst(true);
    const offset = ed.document.offsetAt(ed.selection.active);
    const pathIdx = this.astProvider.findPathForByte(offset);
    if (!pathIdx) {
      this.panel.setStatus('No AST node at cursor');
      return;
    }
    await this.revealAtCursor(true);
    await vscode.commands.executeCommand(`${VIEWS.queryAst}.focus`);
  }

  private async runQuery(): Promise<void> {
    const ed = this.activeEditor();
    if (!ed) {
      this.panel.setStatus('Open a source file to run the query against.');
      return;
    }
    const state = this.panel.getState();
    if (!state.query.trim()) {
      this.panel.setStatus('Query is empty');
      return;
    }
    const rel = this.relativePath(ed.document.uri);
    try {
      const resp = await this.bridge.debugQuery({
        source: ed.document.getText(),
        path: rel,
        query: state.query,
        instrument: true,
      });
      if (!resp.ok || !resp.result) {
        this.panel.setStatus(resp.error ?? 'debugQuery failed');
        this.matches = [];
        this.panel.setMatches([], 0);
        return;
      }
      if (resp.result.hasError) {
        this.panel.setStatus(
          `File has syntax errors — engine may reject edits. Matches: ${resp.result.matchCount}`
        );
      } else {
        this.panel.setStatus(`${resp.result.matchCount} match(es)`);
      }
      this.matches = resp.result.matches;
      this.focusIndex = 0;
      this.panel.setMatches(this.matches, this.focusIndex);
      this.paintMatches(state.capture, state.anchor);
      if (this.isAstViewVisible()) {
        await this.refreshAst(true);
      }
    } catch (e) {
      this.panel.setStatus(e instanceof Error ? e.message : String(e));
    }
  }

  private paintMatches(
    capture: string,
    anchor: 'start' | 'end'
  ): void {
    const ed = this.activeEditor();
    if (!ed) {
      return;
    }
    this.decorations.showMatches(
      ed,
      this.matches,
      this.focusIndex,
      capture || undefined,
      anchor
    );
  }

  private shiftMatch(delta: number): void {
    if (!this.matches.length) {
      return;
    }
    this.focusIndex =
      (this.focusIndex + delta + this.matches.length) % this.matches.length;
    const state = this.panel.getState();
    this.panel.setMatches(this.matches, this.focusIndex);
    this.paintMatches(state.capture, state.anchor);
  }

  private async copy(
    kind: 'query' | 'insert' | 'replace' | 'remove'
  ): Promise<void> {
    const state = this.panel.getState();
    let text = state.query;
    if (kind !== 'query') {
      text = formatYamlOp(kind, state.query, state.capture, state.anchor);
    }
    await vscode.env.clipboard.writeText(text);
    this.panel.setStatus(`Copied ${kind}`);
  }

  async openFromRecipe(
    document?: vscode.TextDocument,
    line?: number
  ): Promise<void> {
    const doc =
      document ?? vscode.window.activeTextEditor?.document;
    if (!doc) {
      return;
    }
    const lineNo = line ?? vscode.window.activeTextEditor?.selection.active.line ?? 0;
    const extracted = extractQueryCaptureAnchor(doc, lineNo);
    if (!extracted?.query) {
      vscode.window.showWarningMessage('Query Tools: could not find a query near the cursor');
      return;
    }
    this.panel.setQuery(
      extracted.query,
      extracted.capture,
      extracted.anchor
    );
    const editPath = findEditPathNearLine(doc, lineNo);
    await vscode.commands.executeCommand(`${VIEWS.queryEditor}.focus`);
    if (editPath) {
      const resolved = await this.bridge.resolveStaticPath(editPath);
      if (resolved.staticResolvable && resolved.path) {
        const abs = path.isAbsolute(resolved.path)
          ? resolved.path
          : path.join(this.config.workspaceRoot, resolved.path);
        const uri = vscode.Uri.file(abs);
        try {
          const opened = await vscode.workspace.openTextDocument(uri);
          await vscode.window.showTextDocument(opened, vscode.ViewColumn.One);
          this.boundUri = uri;
          if (this.isAstViewVisible()) {
            await this.refreshAst(true);
          }
          await this.runQuery();
          return;
        } catch {
          this.panel.setStatus(
            `Loaded query; could not open ${resolved.path}`
          );
          return;
        }
      }
      this.panel.setStatus(
        `Loaded query. Set active editor to the target file (path has unresolved template: ${editPath}).`
      );
      return;
    }
    this.panel.setStatus('Loaded query. Open the target source file to Run.');
  }

  async goToEditPath(
    document?: vscode.TextDocument,
    line?: number
  ): Promise<void> {
    const doc =
      document ?? vscode.window.activeTextEditor?.document;
    if (!doc) {
      return;
    }
    const lineNo = line ?? vscode.window.activeTextEditor?.selection.active.line ?? 0;
    const pathValue = findEditPathNearLine(doc, lineNo);
    if (!pathValue) {
      vscode.window.showWarningMessage('No edit.path found near cursor');
      return;
    }
    const resolved = await this.bridge.resolveStaticPath(pathValue);
    if (!resolved.staticResolvable || !resolved.path) {
      vscode.window.showWarningMessage(
        `edit.path needs parameters: ${resolved.error ?? pathValue}`
      );
      return;
    }
    const abs = path.isAbsolute(resolved.path)
      ? resolved.path
      : path.join(this.config.workspaceRoot, resolved.path);
    const uri = vscode.Uri.file(abs);
    const opened = await vscode.workspace.openTextDocument(uri);
    await vscode.window.showTextDocument(opened);
  }

  dispose(): void {
    if (this.revealTimer) {
      clearTimeout(this.revealTimer);
    }
    this.astProvider.dispose();
    this.panel.dispose();
    this.decorations.dispose();
    for (const d of this.disposables) {
      d.dispose();
    }
  }
}
