import type { FilePreview, SelectionPayload } from '../shared';

export interface FileCardSelection {
  path: string;
  include: boolean;
}

export function defaultFileSelections(files: FilePreview[]): FileCardSelection[] {
  return files.map((file) => ({
    path: file.path,
    include: true,
  }));
}

export function buildSelection(
  fileSelections: FileCardSelection[]
): SelectionPayload {
  const selection: SelectionPayload = { files: {} };
  for (const file of fileSelections) {
    selection.files[file.path] = { include: file.include };
  }
  return selection;
}

export function allFileRows(
  fileSelections: FileCardSelection[]
): { path: string }[] {
  return fileSelections.map((f) => ({ path: f.path }));
}
