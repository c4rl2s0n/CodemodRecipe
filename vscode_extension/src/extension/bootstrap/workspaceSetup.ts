import * as fs from 'fs';
import * as path from 'path';
import * as vscode from 'vscode';

/** True when settings pin an absolute workspace root (no folder required). */
export function configHasAbsoluteWorkspaceRoot(): boolean {
  const configured =
    vscode.workspace
      .getConfiguration('codemodRecipe')
      .get<string>('workspaceRoot') || '';
  return Boolean(configured && path.isAbsolute(configured));
}

/** Empty dir, or tree with no yaml/template recipe assets (depth-capped). */
export function isEmptyCodemodDir(dir: string): boolean {
  try {
    const entries = fs.readdirSync(dir);
    if (entries.length === 0) {
      return true;
    }
    return !walkHasYaml(dir);
  } catch {
    return true;
  }
}

export function walkHasYaml(dir: string, depth = 0): boolean {
  if (depth > 6) {
    return false;
  }
  let entries: string[];
  try {
    entries = fs.readdirSync(dir);
  } catch {
    return false;
  }
  for (const entry of entries) {
    const full = path.join(dir, entry);
    let stat: fs.Stats;
    try {
      stat = fs.statSync(full);
    } catch {
      continue;
    }
    if (stat.isFile() && /\.(ya?ml|template)$/i.test(entry)) {
      return true;
    }
    if (stat.isDirectory() && walkHasYaml(full, depth + 1)) {
      return true;
    }
  }
  return false;
}

/** Prefix host/scaffold errors; hint how to build codemod_host when spawn fails. */
export function formatHostError(message: string): string {
  const prefixed = message.startsWith('Codemod Recipe')
    ? message
    : `Codemod Recipe: ${message}`;
  if (
    /build\.sh|cargo build|codemod_host|Failed to spawn|failed to start|Persistent host/i.test(
      message
    )
  ) {
    if (/build\.sh|cargo/i.test(message)) {
      return prefixed;
    }
    return `${prefixed}\n\nIf the host failed to start, build it via vscode_extension/build.sh or cargo build -p codemod_recipe_host --bin codemod_host.`;
  }
  return prefixed;
}
