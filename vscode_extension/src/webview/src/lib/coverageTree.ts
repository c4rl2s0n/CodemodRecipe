import type { FileCoverage } from '../shared';

export type CoverageTint = 'green' | 'yellow' | 'red';

export interface CoverageTreeNode {
  kind: 'dir' | 'file';
  name: string;
  /** Workspace-relative path for files; dir path prefix without trailing slash. */
  path: string;
  tint: CoverageTint;
  ratio?: number;
  gaps?: Array<{ from: number; to: number }>;
  children?: CoverageTreeNode[];
}

export function fileTint(ratio: number): CoverageTint {
  if (ratio === 1) {
    return 'green';
  }
  if (ratio === 0) {
    return 'red';
  }
  return 'yellow';
}

export function rollupTint(children: readonly CoverageTint[]): CoverageTint {
  if (children.length === 0) {
    return 'yellow';
  }
  if (children.every((t) => t === 'green')) {
    return 'green';
  }
  if (children.every((t) => t === 'red')) {
    return 'red';
  }
  return 'yellow';
}

interface MutableDir {
  kind: 'dir';
  name: string;
  path: string;
  kids: Map<string, MutableDir | MutableFile>;
}

interface MutableFile {
  kind: 'file';
  name: string;
  path: string;
  ratio: number;
  gaps: Array<{ from: number; to: number }>;
}

/** Build a coverage tree from host `files[]` (codebase universe only). */
export function buildCoverageTree(
  files: readonly FileCoverage[]
): CoverageTreeNode[] {
  const root: MutableDir = {
    kind: 'dir',
    name: '',
    path: '',
    kids: new Map(),
  };

  for (const file of files) {
    const parts = file.path.replace(/\\/g, '/').split('/').filter(Boolean);
    if (parts.length === 0) {
      continue;
    }
    let cursor = root;
    for (let i = 0; i < parts.length - 1; i++) {
      const seg = parts[i]!;
      const dirPath = parts.slice(0, i + 1).join('/');
      let next = cursor.kids.get(seg);
      if (!next || next.kind !== 'dir') {
        next = { kind: 'dir', name: seg, path: dirPath, kids: new Map() };
        cursor.kids.set(seg, next);
      }
      cursor = next;
    }
    const fileName = parts[parts.length - 1]!;
    cursor.kids.set(fileName, {
      kind: 'file',
      name: fileName,
      path: file.path.replace(/\\/g, '/'),
      ratio: file.ratio,
      gaps: file.gaps ?? [],
    });
  }

  return finalizeDir(root).children ?? [];
}

function finalizeDir(dir: MutableDir): CoverageTreeNode {
  const children: CoverageTreeNode[] = [];
  const sortedKeys = [...dir.kids.keys()].sort((a, b) => a.localeCompare(b));
  for (const key of sortedKeys) {
    const kid = dir.kids.get(key)!;
    if (kid.kind === 'dir') {
      children.push(finalizeDir(kid));
    } else {
      children.push({
        kind: 'file',
        name: kid.name,
        path: kid.path,
        tint: fileTint(kid.ratio),
        ratio: kid.ratio,
        gaps: kid.gaps,
      });
    }
  }
  return {
    kind: 'dir',
    name: dir.name || '/',
    path: dir.path,
    tint: rollupTint(children.map((c) => c.tint)),
    children,
  };
}
