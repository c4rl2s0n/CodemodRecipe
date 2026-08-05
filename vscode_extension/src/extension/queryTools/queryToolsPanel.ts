import * as vscode from 'vscode';
import type { DebugMatchDto } from '../../shared';

export type QueryToolsPanelState = {
  query: string;
  capture: string;
  anchor: 'start' | 'end';
  pinEq: boolean;
  instrument: boolean;
  status: string;
  stale: boolean;
  captures: string[];
  matches: { index: number; summary: string }[];
  focusIndex: number;
};

export class QueryToolsPanelProvider
  implements vscode.WebviewViewProvider, vscode.Disposable
{
  public static readonly viewType = 'codemodRecipe.queryEditor';

  private view?: vscode.WebviewView;
  private state: QueryToolsPanelState = {
    query: '',
    capture: '',
    anchor: 'end',
    pinEq: false,
    instrument: true,
    status: 'Open a source file, then Generate or paste a query.',
    stale: false,
    captures: [],
    matches: [],
    focusIndex: 0,
  };

  private readonly _onRun = new vscode.EventEmitter<QueryToolsPanelState>();
  readonly onRun = this._onRun.event;
  private readonly _onStateChange = new vscode.EventEmitter<QueryToolsPanelState>();
  readonly onStateChange = this._onStateChange.event;
  private readonly _onCopy = new vscode.EventEmitter<'query' | 'insert' | 'replace' | 'remove'>();
  readonly onCopy = this._onCopy.event;
  private readonly _onPinToggle = new vscode.EventEmitter<boolean>();
  readonly onPinToggle = this._onPinToggle.event;

  resolveWebviewView(webviewView: vscode.WebviewView): void {
    this.view = webviewView;
    webviewView.webview.options = { enableScripts: true };
    webviewView.webview.html = this.html();
    webviewView.webview.onDidReceiveMessage(
      (msg: {
        type: string;
        payload?: Partial<QueryToolsPanelState> & {
          kind?: 'query' | 'insert' | 'replace' | 'remove';
          index?: number;
        };
      }) => {
        if (msg.type === 'state' && msg.payload) {
          const prevPin = this.state.pinEq;
          this.state = { ...this.state, ...msg.payload };
          this.state.captures = extractCaptures(this.state.query);
          if (!this.state.capture && this.state.captures.length) {
            this.state.capture = this.state.captures[this.state.captures.length - 1];
          }
          this._onStateChange.fire(this.state);
          this.postState();
          if (typeof msg.payload.pinEq === 'boolean' && msg.payload.pinEq !== prevPin) {
            this._onPinToggle.fire(this.state.pinEq);
          }
        } else if (msg.type === 'run') {
          this._onRun.fire(this.getState());
        } else if (msg.type === 'copy') {
          this._onCopy.fire(msg.payload?.kind ?? 'query');
        } else if (msg.type === 'focusMatch') {
          const idx = msg.payload?.index ?? 0;
          this.state.focusIndex = idx;
          this.postState();
          this._onStateChange.fire(this.state);
        }
      }
    );
    this.postState();
  }

  getState(): QueryToolsPanelState {
    return { ...this.state };
  }

  setQuery(query: string, capture?: string, anchor?: 'start' | 'end'): void {
    this.state.query = query;
    this.state.captures = extractCaptures(query);
    this.state.capture =
      capture ??
      (this.state.captures.length
        ? this.state.captures[this.state.captures.length - 1]
        : '');
    if (anchor) {
      this.state.anchor = anchor;
    }
    this.postState();
  }

  setStatus(status: string, stale?: boolean): void {
    this.state.status = status;
    if (typeof stale === 'boolean') {
      this.state.stale = stale;
    }
    this.postState();
  }

  setStale(stale: boolean): void {
    this.state.stale = stale;
    this.postState();
  }

  setMatches(matches: DebugMatchDto[], focusIndex: number): void {
    this.state.matches = matches.map((m, i) => ({
      index: i,
      summary: `${m.root.kind} L${m.root.startLine + 1} (${m.captures.length} caps)`,
    }));
    this.state.focusIndex = focusIndex;
    this.postState();
  }

  isPinEq(): boolean {
    return this.state.pinEq;
  }

  isInstrument(): boolean {
    return this.state.instrument;
  }

  private postState(): void {
    void this.view?.webview.postMessage({ type: 'state', payload: this.state });
  }

  private html(): string {
    return `<!DOCTYPE html>
<html><head>
<meta charset="UTF-8"/>
<style>
  body { font-family: var(--vscode-font-family); font-size: 12px; color: var(--vscode-foreground); padding: 8px; }
  textarea { width: 100%; height: 140px; font-family: var(--vscode-editor-font-family); background: var(--vscode-input-background); color: var(--vscode-input-foreground); border: 1px solid var(--vscode-input-border); }
  select, button { margin: 4px 4px 4px 0; }
  .row { display: flex; flex-wrap: wrap; align-items: center; gap: 4px; margin: 6px 0; }
  .status { opacity: 0.85; white-space: pre-wrap; margin-top: 8px; }
  .status.warn { color: var(--vscode-editorWarning-foreground); opacity: 1; font-weight: 600; }
  .matches { list-style: none; padding: 0; margin: 4px 0; max-height: 120px; overflow: auto; }
  .matches li { cursor: pointer; padding: 2px 4px; }
  .matches li.active { background: var(--vscode-list-activeSelectionBackground); color: var(--vscode-list-activeSelectionForeground); }
  label { margin-right: 4px; }
  .hint { opacity: 0.75; font-size: 11px; margin: 4px 0; }
</style>
</head><body>
  <textarea id="query" spellcheck="false" placeholder="(class_definition …) @target"></textarea>
  <div class="row">
    <label>capture</label>
    <select id="capture"></select>
    <label>anchor</label>
    <select id="anchor">
      <option value="start">start</option>
      <option value="end" selected>end</option>
    </select>
    <label title="On generate: add (#eq? @capture &quot;node text&quot;). Does not change Run. Toggling regenerates the current selection."><input type="checkbox" id="pinEq"/> Pin text (#eq?)</label>
    <label title="Inject temporary @__layer_* captures for depth coloring. Turn off if Run fails on queries that use tree-sitter ."><input type="checkbox" id="instrument" checked/> Layer highlights</label>
  </div>
  <p class="hint">Pin text = exact literal on generate (no <code>.</code>). Without Pin, last list items get tree-sitter <code>.</code>. Layer highlights are Run-only visualization.</p>
  <div class="row">
    <button id="run">Run</button>
    <button id="copy">Copy</button>
    <label>Copy as YAML</label>
    <select id="copyYaml">
      <option value="" selected disabled>choose…</option>
      <option value="insert">insert</option>
      <option value="replace">replace</option>
      <option value="remove">remove</option>
    </select>
  </div>
  <ul class="matches" id="matches"></ul>
  <div class="status" id="status"></div>
<script>
  const vscode = acquireVsCodeApi();
  const queryEl = document.getElementById('query');
  const captureEl = document.getElementById('capture');
  const anchorEl = document.getElementById('anchor');
  const pinEqEl = document.getElementById('pinEq');
  const instrumentEl = document.getElementById('instrument');
  const copyYamlEl = document.getElementById('copyYaml');
  const statusEl = document.getElementById('status');
  const matchesEl = document.getElementById('matches');
  let suppress = false;

  function emitState() {
    if (suppress) return;
    vscode.postMessage({
      type: 'state',
      payload: {
        query: queryEl.value,
        capture: captureEl.value,
        anchor: anchorEl.value,
        pinEq: pinEqEl.checked,
        instrument: instrumentEl.checked,
      }
    });
  }

  queryEl.addEventListener('change', emitState);
  queryEl.addEventListener('blur', emitState);
  captureEl.addEventListener('change', emitState);
  anchorEl.addEventListener('change', emitState);
  pinEqEl.addEventListener('change', emitState);
  instrumentEl.addEventListener('change', emitState);
  document.getElementById('run').onclick = () => { emitState(); vscode.postMessage({ type: 'run' }); };
  document.getElementById('copy').onclick = () => vscode.postMessage({ type: 'copy', payload: { kind: 'query' } });
  copyYamlEl.addEventListener('change', () => {
    const kind = copyYamlEl.value;
    if (!kind) return;
    vscode.postMessage({ type: 'copy', payload: { kind } });
    copyYamlEl.selectedIndex = 0;
  });

  window.addEventListener('message', (e) => {
    const msg = e.data;
    if (msg.type !== 'state') return;
    const s = msg.payload;
    suppress = true;
    queryEl.value = s.query || '';
    captureEl.innerHTML = '';
    (s.captures || []).forEach((c) => {
      const o = document.createElement('option');
      o.value = c; o.textContent = c;
      if (c === s.capture) o.selected = true;
      captureEl.appendChild(o);
    });
    anchorEl.value = s.anchor || 'end';
    pinEqEl.checked = !!s.pinEq;
    instrumentEl.checked = s.instrument !== false;
    statusEl.textContent = s.stale
      ? ('⚠ ' + (s.status || 'Stale (unsaved) — save to refresh AST tree.'))
      : (s.status || '');
    statusEl.className = s.stale ? 'status warn' : 'status';
    matchesEl.innerHTML = '';
    (s.matches || []).forEach((m) => {
      const li = document.createElement('li');
      li.textContent = m.summary;
      if (m.index === s.focusIndex) li.className = 'active';
      li.onclick = () => vscode.postMessage({ type: 'focusMatch', payload: { index: m.index } });
      matchesEl.appendChild(li);
    });
    suppress = false;
  });
</script>
</body></html>`;
  }

  dispose(): void {
    this._onRun.dispose();
    this._onStateChange.dispose();
    this._onCopy.dispose();
    this._onPinToggle.dispose();
  }
}

export function extractCaptures(query: string): string[] {
  const names: string[] = [];
  const re = /@([A-Za-z_][\w]*)/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(query))) {
    if (!m[1].startsWith('__layer_') && !names.includes(m[1])) {
      names.push(m[1]);
    }
  }
  return names;
}

export function formatYamlOp(
  kind: 'insert' | 'replace' | 'remove',
  query: string,
  capture: string,
  anchor: 'start' | 'end'
): string {
  const indented = query
    .trim()
    .split('\n')
    .map((l) => `              ${l}`)
    .join('\n');
  if (kind === 'insert') {
    return `        - insert:
            query: |
${indented}
            capture: ${capture || 'target'}
            anchor: ${anchor}
            text: ""
`;
  }
  if (kind === 'replace') {
    return `        - replace:
            query: |
${indented}
            capture: ${capture || 'target'}
            text: ""
`;
  }
  return `        - remove:
            query: |
${indented}
            capture: ${capture || 'target'}
`;
}
