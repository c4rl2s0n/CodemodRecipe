import type { FilePreview, PatchInfo } from '../shared';

export interface LineChangeStats {
  additions: number;
  deletions: number;
}

/** Count logical lines for diff stats (git-style, no trailing-empty split). */
export function countLines(text: string): number {
  if (text === '') {
    return 0;
  }
  const normalized = text.replace(/\r\n/g, '\n');
  const parts = normalized.split('\n');
  if (parts.length > 1 && parts[parts.length - 1] === '') {
    parts.pop();
  }
  return parts.length;
}

function normalizedLines(text: string): string[] {
  if (text === '') {
    return [];
  }
  const normalized = text.replace(/\r\n/g, '\n');
  const parts = normalized.split('\n');
  if (parts.length > 1 && parts[parts.length - 1] === '') {
    parts.pop();
  }
  return parts;
}

function lcsLength(a: string[], b: string[]): number {
  if (!a.length || !b.length) {
    return 0;
  }
  const prev = new Array<number>(b.length + 1).fill(0);
  const curr = new Array<number>(b.length + 1).fill(0);
  for (let i = 1; i <= a.length; i++) {
    for (let j = 1; j <= b.length; j++) {
      if (a[i - 1] === b[j - 1]) {
        curr[j] = prev[j - 1] + 1;
      } else {
        curr[j] = Math.max(prev[j], curr[j - 1]);
      }
    }
    for (let j = 0; j <= b.length; j++) {
      prev[j] = curr[j];
      curr[j] = 0;
    }
  }
  return prev[b.length];
}

/** Line additions/deletions using line-wise diff semantics (git-like counts). */
export function lineChangeStats(removed: string, added: string): LineChangeStats {
  const removedLines = normalizedLines(removed);
  const addedLines = normalizedLines(added);
  const common = lcsLength(removedLines, addedLines);
  return {
    additions: Math.max(0, addedLines.length - common),
    deletions: Math.max(0, removedLines.length - common),
  };
}

export function statsForPatch(
  file: FilePreview,
  patch: PatchInfo
): LineChangeStats {
  if (patch.index < 0) {
    return statsForFile(file);
  }
  const original = file.original ?? '';
  const removed = original.slice(patch.offset, patch.offset + patch.length);
  const added =
    patch.replacement ??
    patch.replacementPreview ??
    '';
  return lineChangeStats(removed, added);
}

export function statsForFile(file: FilePreview): LineChangeStats {
  return lineChangeStats(file.original ?? '', file.modified ?? '');
}

export function sumStats(stats: LineChangeStats[]): LineChangeStats {
  let additions = 0;
  let deletions = 0;
  for (const s of stats) {
    additions += s.additions;
    deletions += s.deletions;
  }
  return { additions, deletions };
}
