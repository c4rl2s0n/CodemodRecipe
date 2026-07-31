import * as vscode from 'vscode';
import {
  buildUriContextValues,
  prefillArgsFromUriClick,
  toWorkspaceRelativePath,
  type ExplorerResourceKind,
  type UriContextValues,
} from './recipeUriContext';

export type { ExplorerResourceKind, UriContextValues as UriContext };
export { prefillArgsFromUriClick, toWorkspaceRelativePath };

export interface EditorContext {
  readonly values: Record<string, string>;
  readonly source: string;
  readonly languageId: string;
  readonly cursorOffset: number;
  readonly selectionStart: number;
  readonly selectionEnd: number;
  readonly filePath: string;
}

export {
  argNeedsHostDerive,
  effectiveFrom,
  mergeArgLayers,
  missingRequiredArgNames,
  prefillArgs,
  recipeNeedsHostDerive,
  renderContextTemplate,
} from './recipeContextValues';

/** Build context builtins from an Explorer file/folder URI. */
export function resolveUriContext(
  uri: vscode.Uri,
  workspaceRoot: string,
  kind: ExplorerResourceKind
): UriContextValues {
  const relative = toWorkspaceRelativePath(workspaceRoot, uri.fsPath);
  return buildUriContextValues(relative, kind, workspaceRoot);
}

export function resolveEditorContext(workspaceRoot: string): EditorContext {
  const editor = vscode.window.activeTextEditor;
  if (!editor) {
    return {
      values: {},
      source: '',
      languageId: '',
      cursorOffset: 0,
      selectionStart: 0,
      selectionEnd: 0,
      filePath: '',
    };
  }

  const document = editor.document;
  const selection = document.getText(editor.selection);
  const wordRange = document.getWordRangeAtPosition(editor.selection.active);
  const word = wordRange ? document.getText(wordRange) : '';
  const file = toWorkspaceRelativePath(workspaceRoot, document.uri.fsPath);
  const parts = buildUriContextValues(file, 'file').values;
  const activeLine = editor.selection.active.line;
  const line = document.lineAt(activeLine).text;
  const cursorOffset = document.offsetAt(editor.selection.active);
  const selectionStart = document.offsetAt(editor.selection.start);
  const selectionEnd = document.offsetAt(editor.selection.end);

  const values: Record<string, string> = {
    file,
    fileBasename: parts.fileBasename,
    fileDirname: parts.fileDirname,
    fileStem: parts.fileStem,
    fileExt: parts.fileExt,
    selection,
    word,
    line,
    lineNumber: String(activeLine + 1),
    languageId: document.languageId,
  };

  return {
    values,
    source: document.getText(),
    languageId: document.languageId,
    cursorOffset,
    selectionStart,
    selectionEnd,
    filePath: file,
  };
}
