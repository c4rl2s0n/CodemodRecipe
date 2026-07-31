import * as path from 'path';
import * as vscode from 'vscode';

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
  const relativePath = path.relative(workspaceRoot, document.uri.fsPath);
  const file = relativePath.startsWith('..')
    ? document.uri.fsPath
    : relativePath;
  const fileBasename = path.basename(file);
  const fileExt = path.extname(fileBasename);
  const fileStem = fileExt
    ? fileBasename.slice(0, -fileExt.length)
    : fileBasename;
  const fileDirname = path.dirname(file).replace(/\\/g, '/');
  const activeLine = editor.selection.active.line;
  const line = document.lineAt(activeLine).text;
  const cursorOffset = document.offsetAt(editor.selection.active);
  const selectionStart = document.offsetAt(editor.selection.start);
  const selectionEnd = document.offsetAt(editor.selection.end);

  const values: Record<string, string> = {
    file,
    fileBasename,
    fileDirname,
    fileStem,
    fileExt: fileExt.replace(/^\./, ''),
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
