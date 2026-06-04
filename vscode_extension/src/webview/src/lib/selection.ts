import type { FilePreview, SelectionPayload } from '../shared';

export interface PatchSelection {
  path: string;
  index: number;
  include: boolean;
}

export interface FileCardSelection {
  path: string;
  include: boolean;
  patches: PatchSelection[];
}

export function fileIncludeFromPatches(patches: readonly PatchSelection[]): boolean {
  return patches.some((p) => p.include);
}

/** File toggle: set file include and align every patch. */
export function setFileInclude(
  selection: FileCardSelection,
  include: boolean
): FileCardSelection {
  return {
    ...selection,
    include,
    patches: selection.patches.map((p) => ({ ...p, include })),
  };
}

/** Patch toggle: update one patch and derive file include from all patches. */
export function setPatchInclude(
  selection: FileCardSelection,
  patchIndex: number,
  include: boolean
): FileCardSelection {
  const patches = selection.patches.map((p) =>
    p.index === patchIndex ? { ...p, include } : p
  );
  return {
    ...selection,
    include: fileIncludeFromPatches(patches),
    patches,
  };
}

export function defaultFileSelections(files: FilePreview[]): FileCardSelection[] {
  return files.map((file) => {
    const patches =
      file.patches.length > 0
        ? file.patches.map((patch) => ({
            path: file.path,
            index: patch.index,
            include: true,
          }))
        : [
            {
              path: file.path,
              index: -1,
              include: true,
            },
          ];
    return {
      path: file.path,
      include: true,
      patches,
    };
  });
}

export function buildSelection(
  fileSelections: FileCardSelection[]
): SelectionPayload {
  const selection: SelectionPayload = { files: {} };
  for (const file of fileSelections) {
    const patchToggles = file.patches.filter((p) => p.index >= 0);
    const entry: { include: boolean; patches?: number[] } = {
      include: file.include,
    };
    if (patchToggles.length > 0) {
      entry.patches = patchToggles.filter((p) => p.include).map((p) => p.index);
    }
    selection.files[file.path] = entry;
  }
  return selection;
}

export function allPatchRows(
  fileSelections: FileCardSelection[]
): { path: string; index: number }[] {
  const rows: { path: string; index: number }[] = [];
  for (const file of fileSelections) {
    for (const patch of file.patches) {
      rows.push({ path: patch.path, index: patch.index });
    }
  }
  return rows;
}
