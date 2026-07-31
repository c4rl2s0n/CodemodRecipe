import * as fs from 'fs';
import * as vscode from 'vscode';

export type SurfaceContainer = {
  children: string[];
  mapValue?: boolean;
  scalarAlt?: boolean;
};

export type DslSurface = {
  version: number;
  containers: Record<string, SurfaceContainer>;
  enums: Record<string, string[]>;
  parentToContainer: Record<string, string>;
  documentRoots: Record<string, string>;
};

let cached: DslSurface | undefined;

export function loadDslSurface(extensionUri: vscode.Uri): DslSurface {
  if (cached) {
    return cached;
  }
  const file = vscode.Uri.joinPath(
    extensionUri,
    'schemas',
    'generated-dsl-surface.json'
  );
  try {
    const raw = fs.readFileSync(file.fsPath, 'utf8');
    cached = JSON.parse(raw) as DslSurface;
  } catch {
    cached = {
      version: 0,
      containers: {},
      enums: {},
      parentToContainer: { '': 'recipeRoot' },
      documentRoots: { recipe: 'recipeRoot' },
    };
  }
  return cached;
}

/** Test helper / reset after codegen in tests. */
export function resetDslSurfaceCache(): void {
  cached = undefined;
}

export function resolveContainerId(
  surface: DslSurface,
  parentWire: string | undefined,
  path: string[]
): string | undefined {
  if (parentWire === undefined || parentWire === '') {
    // Heuristic: map/variables assets vs recipe
    if (path.length === 0) {
      return surface.documentRoots.recipe ?? 'recipeRoot';
    }
  }
  if (parentWire && surface.parentToContainer[parentWire]) {
    return surface.parentToContainer[parentWire];
  }
  // Nested under recipe: with no parent → recipeRoot
  if (!parentWire) {
    return surface.documentRoots.recipe ?? 'recipeRoot';
  }
  return undefined;
}

export function remainingChildKeys(
  surface: DslSurface,
  containerId: string,
  siblingKeys: readonly string[]
): string[] {
  const container = surface.containers[containerId];
  if (!container || container.mapValue) {
    return [];
  }
  const present = new Set(siblingKeys);
  return container.children.filter((child) => !present.has(child));
}
