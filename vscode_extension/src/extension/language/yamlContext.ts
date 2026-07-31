/**
 * Indent-stack YAML context for recipe documents (pure; no vscode types).
 */

export type YamlPositionKind = 'key' | 'value' | 'unknown';

export type YamlContext = {
  positionKind: YamlPositionKind;
  /** Nearest key at strictly lower indent, if any. */
  parentWire?: string;
  siblingKeys: string[];
  /** Approximate key path from document root. */
  path: string[];
  currentLinePrefix: string;
  /** Indent column of the current line (spaces). */
  indent: number;
};

export type LineSource = {
  lineCount: number;
  lineText(line: number): string;
};

/** Adapt a VS Code TextDocument-like object. */
export function lineSourceFromDocument(document: {
  lineCount: number;
  lineAt(line: number): { text: string };
}): LineSource {
  return {
    lineCount: document.lineCount,
    lineText(line: number) {
      return document.lineAt(line).text;
    },
  };
}

function lineIndent(text: string): number {
  const match = text.match(/^(\s*)/);
  return match?.[1].length ?? 0;
}

function keyAtLine(text: string): string | undefined {
  const trimmed = text.trim();
  if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith('-')) {
    const listKey = text.match(/^\s*-\s*([A-Za-z_][\w]*)\s*:/);
    if (listKey) {
      return listKey[1];
    }
    return undefined;
  }
  const match = text.match(/^(\s*)([A-Za-z_][\w]*)\s*:/);
  return match?.[2];
}

function isListItemDiscriminator(text: string): string | undefined {
  const match = text.match(/^\s*-\s*([A-Za-z_][\w]*)\s*:/);
  return match?.[1];
}

/**
 * Resolve YAML editor context at a 0-based line / character offset.
 */
export function resolveYamlContext(
  document: LineSource,
  line: number,
  character: number
): YamlContext {
  const safeLine = Math.max(0, Math.min(line, document.lineCount - 1));
  const text = document.lineText(safeLine);
  const before = text.slice(0, Math.max(0, character));
  const indent = lineIndent(text);

  const path: string[] = [];
  const stack: { indent: number; wire: string }[] = [];

  for (let i = 0; i < safeLine; i++) {
    const lt = document.lineText(i);
    if (lt.trim() === '' || lt.trim().startsWith('#')) {
      continue;
    }
    const ind = lineIndent(lt);
    const disc = isListItemDiscriminator(lt);
    const key = disc ?? keyAtLine(lt);
    if (!key) {
      continue;
    }
    while (stack.length > 0 && stack[stack.length - 1].indent >= ind) {
      stack.pop();
    }
    // List discriminators (`- edit:`) are parents for following indented fields.
    const valueEmpty = /:\s*$/.test(lt.trim()) || disc !== undefined;
    if (valueEmpty || disc) {
      stack.push({ indent: ind, wire: key });
    }
  }

  // Adjust stack for current line indent.
  while (stack.length > 0 && stack[stack.length - 1].indent >= indent) {
    stack.pop();
  }

  for (const frame of stack) {
    path.push(frame.wire);
  }

  const parentWire = stack.length > 0 ? stack[stack.length - 1].wire : undefined;
  const parentIndent = stack.length > 0 ? stack[stack.length - 1].indent : -1;

  const siblingKeys: string[] = [];
  if (parentWire !== undefined || indent === 0) {
    for (let i = 0; i < document.lineCount; i++) {
      const lt = document.lineText(i);
      if (lt.trim() === '') {
        continue;
      }
      const ind = lineIndent(lt);
      if (parentIndent < 0) {
        if (ind !== 0) {
          continue;
        }
      } else if (ind <= parentIndent) {
        if (i > safeLine) {
          break;
        }
        continue;
      } else if (ind !== indent && !(indent === 0 && ind === 0)) {
        // Only same-indent siblings under parent.
        if (parentIndent >= 0 && ind !== parentIndent + 2 && ind !== indent) {
          // Allow common 2-space step; still collect exact indent matches.
          if (ind !== indent) {
            continue;
          }
        } else if (ind !== indent) {
          continue;
        }
      }
      if (ind !== indent) {
        continue;
      }
      // Must still be under same parent: walk back to ensure parent line.
      if (parentIndent >= 0) {
        let ok = false;
        for (let j = i - 1; j >= 0; j--) {
          const prev = document.lineText(j);
          if (prev.trim() === '') {
            continue;
          }
          const pInd = lineIndent(prev);
          if (pInd < indent) {
            const pKey = isListItemDiscriminator(prev) ?? keyAtLine(prev);
            ok = pInd === parentIndent && pKey === parentWire;
            break;
          }
        }
        if (!ok) {
          continue;
        }
      }
      const k = isListItemDiscriminator(lt) ?? keyAtLine(lt);
      if (k) {
        siblingKeys.push(k);
      }
    }
  }

  let positionKind: YamlPositionKind = 'unknown';
  const trimmedBefore = before.trimEnd();
  if (/^\s*$/.test(before) || /:\s*$/.test(trimmedBefore) === false && /^\s*[A-Za-z_][\w]*$/.test(trimmedBefore)) {
    // Blank indent or typing a key name.
    if (/^\s*[A-Za-z_][\w]*$/.test(before.trim()) || /^\s*$/.test(before)) {
      positionKind = 'key';
    }
  }
  if (/:\s+\S*$/.test(before) || /:\s*$/.test(before) && character > before.indexOf(':')) {
    const afterColon = before.slice(before.indexOf(':') + 1);
    if (/^\s*['"]?[\w./-]*$/.test(afterColon) || afterColon.trim() === '') {
      positionKind = before.includes(':') && !/^\s*[A-Za-z_][\w]*:\s*$/.test(before.trim())
        ? /:\s*$/.test(before.trimEnd()) && /^\s*[A-Za-z_][\w]*:\s*$/.test(text.trim())
          ? 'key' // `key:` alone on line waiting for nested content — treat as key context for newline children
          : 'value'
        : positionKind;
    }
  }
  // `key: value` on same line → value
  if (/^\s*[A-Za-z_][\w]*:\s+\S/.test(before)) {
    positionKind = 'value';
  }
  // Blank or partial key on indented line under a parent → key
  if (
    (positionKind === 'unknown' || positionKind === 'key') &&
    (/^\s*$/.test(before) || /^\s+[A-Za-z_][\w]*$/.test(before))
  ) {
    positionKind = 'key';
  }
  // Top-level blank → key
  if (/^\s*$/.test(before) && indent === 0) {
    positionKind = 'key';
  }

  return {
    positionKind,
    parentWire,
    siblingKeys,
    path,
    currentLinePrefix: before,
    indent,
  };
}
