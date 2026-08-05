import * as vscode from 'vscode';
import type { AstNodeDto, CaptureInfoDto, DebugMatchDto } from '../../shared';

const DEPTH_COLORS = [
  'rgba(80, 140, 220, 0.18)',
  'rgba(80, 180, 120, 0.18)',
  'rgba(220, 160, 60, 0.18)',
  'rgba(180, 100, 200, 0.18)',
  'rgba(200, 90, 90, 0.18)',
  'rgba(60, 180, 180, 0.18)',
];

export class QueryToolsDecorations implements vscode.Disposable {
  private readonly rootType: vscode.TextEditorDecorationType;
  private readonly layerTypes: vscode.TextEditorDecorationType[];
  private readonly captureType: vscode.TextEditorDecorationType;
  private readonly editCaptureType: vscode.TextEditorDecorationType;
  private readonly anchorType: vscode.TextEditorDecorationType;
  private readonly selectionType: vscode.TextEditorDecorationType;
  private readonly disposables: vscode.Disposable[] = [];

  constructor() {
    this.rootType = vscode.window.createTextEditorDecorationType({
      backgroundColor: 'rgba(100, 140, 255, 0.12)',
      overviewRulerColor: 'rgba(100, 140, 255, 0.6)',
      overviewRulerLane: vscode.OverviewRulerLane.Center,
    });
    this.layerTypes = DEPTH_COLORS.map((c) =>
      vscode.window.createTextEditorDecorationType({
        backgroundColor: c,
      })
    );
    this.captureType = vscode.window.createTextEditorDecorationType({
      borderWidth: '1px',
      borderStyle: 'solid',
      borderColor: 'rgba(80, 180, 120, 0.8)',
      overviewRulerColor: 'rgba(80, 180, 120, 0.8)',
      overviewRulerLane: vscode.OverviewRulerLane.Right,
    });
    this.editCaptureType = vscode.window.createTextEditorDecorationType({
      borderWidth: '2px',
      borderStyle: 'solid',
      borderColor: new vscode.ThemeColor('editorWarning.foreground'),
      overviewRulerColor: new vscode.ThemeColor('editorWarning.foreground'),
      overviewRulerLane: vscode.OverviewRulerLane.Full,
    });
    this.anchorType = vscode.window.createTextEditorDecorationType({
      borderWidth: '0 2px 0 0',
      borderStyle: 'solid',
      borderColor: new vscode.ThemeColor('editorInfo.foreground'),
    });
    this.selectionType = vscode.window.createTextEditorDecorationType({
      backgroundColor: 'rgba(255, 200, 0, 0.2)',
    });
    this.disposables.push(
      this.rootType,
      ...this.layerTypes,
      this.captureType,
      this.editCaptureType,
      this.anchorType,
      this.selectionType
    );
  }

  clear(editor: vscode.TextEditor): void {
    editor.setDecorations(this.rootType, []);
    for (const t of this.layerTypes) {
      editor.setDecorations(t, []);
    }
    editor.setDecorations(this.captureType, []);
    editor.setDecorations(this.editCaptureType, []);
    editor.setDecorations(this.anchorType, []);
    editor.setDecorations(this.selectionType, []);
  }

  showAstSelection(editor: vscode.TextEditor, node: AstNodeDto): void {
    this.clear(editor);
    const range = byteRangeToVsRange(editor.document, node.start.byte, node.end.byte);
    editor.setDecorations(this.selectionType, [range]);
    editor.revealRange(range, vscode.TextEditorRevealType.InCenterIfOutsideViewport);
  }

  showMatches(
    editor: vscode.TextEditor,
    matches: DebugMatchDto[],
    focusIndex: number,
    editCapture: string | undefined,
    anchor: 'start' | 'end' | undefined
  ): void {
    this.clear(editor);
    if (matches.length === 0) {
      return;
    }
    const focus = matches[Math.min(focusIndex, matches.length - 1)];
    const rootRange = byteRangeToVsRange(
      editor.document,
      focus.root.start,
      focus.root.end
    );
    editor.setDecorations(this.rootType, [
      {
        range: rootRange,
        hoverMessage: new vscode.MarkdownString(
          `**match root** \`${focus.root.kind}\``
        ),
      },
    ]);

    const layerBuckets: vscode.DecorationOptions[][] = this.layerTypes.map(
      () => []
    );
    const authorCaps: vscode.DecorationOptions[] = [];
    const editCaps: vscode.DecorationOptions[] = [];
    const anchors: vscode.DecorationOptions[] = [];

    const applyCaps = (caps: CaptureInfoDto[], dim: boolean) => {
      for (const cap of caps) {
        const range = byteRangeToVsRange(editor.document, cap.start, cap.end);
        const hover = new vscode.MarkdownString(
          `**@${cap.name}** \`${cap.kind}\`  \nbytes ${cap.start}–${cap.end}`
        );
        const opt: vscode.DecorationOptions = { range, hoverMessage: hover };
        if (cap.isLayer) {
          const idx = Math.min(
            Math.max(cap.depth - 1, 0),
            this.layerTypes.length - 1
          );
          if (!dim) {
            layerBuckets[idx].push(opt);
          }
        } else if (editCapture && cap.name === editCapture) {
          editCaps.push(opt);
          if (anchor) {
            const pos =
              anchor === 'start'
                ? range.start
                : range.end;
            anchors.push({
              range: new vscode.Range(pos, pos),
              hoverMessage: new vscode.MarkdownString(`insert **anchor: ${anchor}**`),
            });
          }
        } else if (!dim) {
          authorCaps.push(opt);
        }
      }
    };

    matches.forEach((m, i) => applyCaps(m.captures, i !== focusIndex));

    for (let i = 0; i < this.layerTypes.length; i++) {
      editor.setDecorations(this.layerTypes[i], layerBuckets[i]);
    }
    editor.setDecorations(this.captureType, authorCaps);
    editor.setDecorations(this.editCaptureType, editCaps);
    editor.setDecorations(this.anchorType, anchors);
    editor.revealRange(rootRange, vscode.TextEditorRevealType.InCenterIfOutsideViewport);
  }

  dispose(): void {
    for (const d of this.disposables) {
      d.dispose();
    }
  }
}

export function byteRangeToVsRange(
  document: vscode.TextDocument,
  start: number,
  end: number
): vscode.Range {
  const s = Math.max(0, Math.min(start, document.offsetAt(document.lineAt(document.lineCount - 1).range.end)));
  const e = Math.max(s, Math.min(end, document.getText().length));
  return new vscode.Range(document.positionAt(s), document.positionAt(e));
}
