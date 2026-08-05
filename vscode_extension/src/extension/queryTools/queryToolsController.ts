import * as path from 'path';
import * as vscode from 'vscode';
import { COMMANDS, VIEWS } from '../constants';
import type { ExtensionConfig } from '../config/extensionConfig';
import type { HostBridge } from '../host/hostBridge';
import type { DebugMatchDto } from '../../shared';
import {
  AstDisplayMode,
  AstTreeItem,
  AstTreeProvider,
  AST_DISPLAY_MODE_SETTING,
  readAstDisplayMode,
} from './astTreeProvider';
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
  private lastGenerateRange: { start: number; end: number } | undefined;

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
          void this.revealMatchInAst(this.focusIndex).then(() => {
            this.paintMatches(s.capture, s.anchor);
          });
        }
      }),
      this.panel.onPinToggle(() => {
        void this.regenerateFromLastOrCursor();
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
      vscode.commands.registerCommand(
        COMMANDS.queryToolsAstDisplayMode,
        () => void this.pickAstDisplayMode()
      ),
      this.astTreeView.onDidChangeVisibility((e) => {
        if (e.visible) {
          void this.refreshAstIfVisible(true);
        } else {
          this.clearEditorDecorations();
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
      }),
      vscode.workspace.onDidChangeConfiguration((e) => {
        if (e.affectsConfiguration(AST_DISPLAY_MODE_SETTING)) {
          this.astProvider.refreshDisplayFromSettings();
        }
      })
    );
    context.subscriptions.push(this);
  }

  private isAstViewVisible(): boolean {
    return this.astTreeView?.visible ?? false;
  }

  private clearEditorDecorations(): void {
    const ed = this.activeEditor();
    if (ed) {
      this.decorations.clear(ed);
    }
  }

  private scheduleRevealAtCursor(): void {
    if (!this.isAstViewVisible() || this.astStale) {
      return;
    }
    if (this.revealTimer) {
      clearTimeout(this.revealTimer);
    }
    this.revealTimer = setTimeout(() => {
      void this.revealAndGenerateAtCursor(false);
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
    this.panel.setStatus(base ? `${stale}\n${base}` : stale, true);
    if (this.astTreeView) {
      this.astTreeView.description = '$(warning) Stale (unsaved)';
      this.astTreeView.message = '⚠ Stale (unsaved) — save to refresh AST';
    }
  }

  private clearStaleStatus(): void {
    this.astStale = false;
    this.panel.setStale(false);
    if (this.astTreeView) {
      this.astTreeView.description = undefined;
      this.astTreeView.message = undefined;
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
        this.panel.setStatus('AST ready — select a node or move the cursor.');
      }
    } catch (e) {
      this.panel.setStatus(e instanceof Error ? e.message : String(e));
    }
  }

  private async revealAtCursor(focusAst: boolean): Promise<AstTreeItem | undefined> {
    const ed = this.activeEditor();
    if (!ed || !this.astProvider.getRoot()) {
      return undefined;
    }
    const offset = ed.document.offsetAt(ed.selection.active);
    const pathIdx = this.astProvider.findPathForByte(offset);
    if (!pathIdx) {
      return undefined;
    }
    const item = this.astProvider.getItemForPath(pathIdx);
    if (!item) {
      return undefined;
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
    return item;
  }

  private async revealAndGenerateAtCursor(focusAst: boolean): Promise<void> {
    const item = await this.revealAtCursor(focusAst);
    if (item) {
      await this.generateForNode(item.node.start.byte, item.node.end.byte, item.node.kind);
    }
  }

  private async generateForNode(
    start: number,
    end: number,
    kindHint?: string
  ): Promise<void> {
    const ed = this.activeEditor();
    if (!ed) {
      return;
    }
    const pinEq = this.panel.isPinEq();
    const rel = this.relativePath(ed.document.uri);
    try {
      const resp = await this.bridge.generateQuery({
        source: ed.document.getText(),
        path: rel,
        start,
        end,
        includeTextPredicates: pinEq,
        captureLeaf: 'target',
        maxDepth: 8,
      });
      if (!resp.ok || !resp.query) {
        this.panel.setStatus(resp.error ?? 'generateQuery failed');
        return;
      }
      this.lastGenerateRange = { start, end };
      this.panel.setQuery(resp.query, resp.captureSuggestion);
      this.panel.setStatus(
        kindHint
          ? `Generated query for ${kindHint}`
          : `Regenerated with Pin text ${pinEq ? 'on' : 'off'}`
      );
    } catch (e) {
      this.panel.setStatus(e instanceof Error ? e.message : String(e));
    }
  }

  private async regenerateFromLastOrCursor(): Promise<void> {
    if (this.lastGenerateRange) {
      await this.generateForNode(
        this.lastGenerateRange.start,
        this.lastGenerateRange.end
      );
      return;
    }
    const ed = this.activeEditor();
    if (!ed) {
      return;
    }
    const offset = ed.document.offsetAt(ed.selection.active);
    await this.generateForNode(offset, offset);
  }

  private async onSelectAstNode(item: AstTreeItem): Promise<void> {
    const ed = this.activeEditor();
    if (!ed) {
      return;
    }
    this.decorations.showAstSelection(ed, item.node);
    await this.generateForNode(
      item.node.start.byte,
      item.node.end.byte,
      item.node.kind
    );
  }

  private async generateFromCursor(): Promise<void> {
    const ed = this.activeEditor();
    if (!ed) {
      vscode.window.showWarningMessage('Query Tools: no active editor');
      return;
    }
    const offset = ed.document.offsetAt(ed.selection.active);
    await this.generateForNode(offset, offset);
    this.panel.setStatus('Generated from cursor');
    if (this.isAstViewVisible()) {
      await this.refreshAst(true);
      await this.revealAtCursor(false);
    }
    await vscode.commands.executeCommand(`${VIEWS.queryEditor}.focus`);
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
    await this.revealAndGenerateAtCursor(true);
    await vscode.commands.executeCommand(`${VIEWS.queryAst}.focus`);
  }

  private async revealMatchInAst(index: number): Promise<void> {
    if (!this.isAstViewVisible() || this.astStale || !this.matches.length) {
      return;
    }
    const match = this.matches[Math.min(index, this.matches.length - 1)];
    if (!match) {
      return;
    }
    const pathIdx = this.astProvider.findPathForByte(match.root.start);
    if (!pathIdx) {
      return;
    }
    const item = this.astProvider.getItemForPath(pathIdx);
    if (!item) {
      return;
    }
    try {
      await this.astTreeView?.reveal(item, {
        expand: true,
        select: true,
        focus: false,
      });
    } catch {
      // ignore
    }
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
        instrument: this.panel.isInstrument(),
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
        await this.revealMatchInAst(0);
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
    void this.revealMatchInAst(this.focusIndex).then(() => {
      this.paintMatches(state.capture, state.anchor);
    });
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

  private async pickAstDisplayMode(): Promise<void> {
    const current = readAstDisplayMode();
    const pick = await vscode.window.showQuickPick(
      [
        {
          label: 'Kind + preview',
          description: 'kind in label, muted text snippet',
          mode: 'kindPreview' as AstDisplayMode,
        },
        {
          label: 'Kind only',
          description: 'kind + line range',
          mode: 'kind' as AstDisplayMode,
        },
        {
          label: 'Content',
          description: 'muted text snippet; kind on hover',
          mode: 'content' as AstDisplayMode,
        },
      ],
      {
        title: 'Query AST display',
        placeHolder: `Current: ${current}`,
      }
    );
    if (!pick) {
      return;
    }
    await vscode.workspace
      .getConfiguration('codemodRecipe.queryTools')
      .update('astDisplayMode', pick.mode, vscode.ConfigurationTarget.Global);
    this.astProvider.setDisplayMode(pick.mode);
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
