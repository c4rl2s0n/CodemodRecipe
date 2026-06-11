import * as path from 'path';
import * as vscode from 'vscode';
import type { RecipeSchema } from '../../shared';

export interface EditorContext {
  readonly values: Record<string, string>;
}

export function resolveEditorContext(workspaceRoot: string): EditorContext {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return { values: {} };

  const document = editor.document;
  const selection = document.getText(editor.selection);
  const wordRange = document.getWordRangeAtPosition(editor.selection.active);
  const word = wordRange ? document.getText(wordRange) : '';
  const relativePath = path.relative(workspaceRoot, document.uri.fsPath);
  const file = relativePath.startsWith('..') ? document.uri.fsPath : relativePath;
  const dartClass = findEnclosingDartClass(
    document.getText(),
    document.offsetAt(editor.selection.active)
  );

  return {
    values: {
      file,
      selection,
      word,
      dartClass: dartClass ?? '',
    },
  };
}

export function prefillArgs(
  recipe: RecipeSchema,
  contextValues: Record<string, string>
): Record<string, string> {
  const args: Record<string, string> = {};
  for (const arg of recipe.args) {
    const contextKey = arg.contextKey;
    if (!contextKey) continue;
    const value = contextValues[contextKey];
    if (value) {
      args[arg.name] = value;
    }
  }
  return args;
}

function findEnclosingDartClass(source: string, offset: number): string | undefined {
  const classPattern = /\bclass\s+([A-Za-z_]\w*)[^{]*\{/g;
  let match: RegExpExecArray | null;
  let best: string | undefined;

  while ((match = classPattern.exec(source)) !== null) {
    const openBrace = source.indexOf('{', match.index);
    if (openBrace === -1 || openBrace > offset) continue;
    const closeBrace = findMatchingBrace(source, openBrace);
    if (closeBrace >= offset) {
      best = match[1];
    }
  }

  return best;
}

function findMatchingBrace(source: string, openBrace: number): number {
  let depth = 0;
  for (let i = openBrace; i < source.length; i++) {
    const char = source[i];
    if (char === '{') depth++;
    if (char === '}') {
      depth--;
      if (depth === 0) return i;
    }
  }
  return source.length;
}
