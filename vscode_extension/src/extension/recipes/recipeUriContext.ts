import * as path from 'path';

export type ExplorerResourceKind = 'file' | 'folder';

export interface UriContextValues {
  readonly kind: ExplorerResourceKind;
  readonly path: string;
  readonly values: Record<string, string>;
}

function pathParts(relativePath: string): {
  fileBasename: string;
  fileStem: string;
  fileExt: string;
  fileDirname: string;
} {
  const normalized = relativePath.replace(/\\/g, '/');
  const fileBasename = path.basename(normalized);
  const fileExtRaw = path.extname(fileBasename);
  const fileStem = fileExtRaw
    ? fileBasename.slice(0, -fileExtRaw.length)
    : fileBasename;
  const fileDirname = path.dirname(normalized).replace(/\\/g, '/');
  return {
    fileBasename,
    fileStem,
    fileExt: fileExtRaw.replace(/^\./, ''),
    fileDirname: fileDirname === '.' ? '' : fileDirname,
  };
}

export function toWorkspaceRelativePath(
  workspaceRoot: string,
  fsPath: string
): string {
  const relativePath = path.relative(workspaceRoot, fsPath);
  if (relativePath.startsWith('..') || path.isAbsolute(relativePath)) {
    return fsPath.replace(/\\/g, '/');
  }
  return relativePath.replace(/\\/g, '/');
}

/** Build context builtins from an Explorer file/folder path. */
export function buildUriContextValues(
  relativePath: string,
  kind: ExplorerResourceKind
): UriContextValues {
  const relative = relativePath.replace(/\\/g, '/');
  const parts = pathParts(relative);
  const values: Record<string, string> = {
    path: relative,
    fileBasename: parts.fileBasename,
    fileStem: parts.fileStem,
    fileExt: parts.fileExt,
    fileDirname: parts.fileDirname,
  };
  if (kind === 'file') {
    values.file = relative;
    values.directory = parts.fileDirname;
  } else {
    values.directory = relative;
    values.fileDirname = relative;
  }
  return { kind, path: relative, values };
}

/** Prefill the first matching `inputKind` arg from the Explorer click path. */
export function prefillArgsFromUriClick(
  recipe: { args: readonly { name: string; inputKind: string }[] },
  kind: ExplorerResourceKind,
  clickPath: string
): Record<string, string> {
  const want = kind === 'folder' ? 'directory' : 'file';
  const target = recipe.args.find((a) => a.inputKind === want);
  if (!target) {
    return {};
  }
  return { [target.name]: clickPath };
}
