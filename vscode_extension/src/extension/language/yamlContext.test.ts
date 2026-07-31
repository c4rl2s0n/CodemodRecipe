import { describe, expect, it } from 'vitest';
import { resolveYamlContext, type LineSource } from './yamlContext';

function doc(lines: string[]): LineSource {
  return {
    lineCount: lines.length,
    lineText(line: number) {
      return lines[line] ?? '';
    },
  };
}

describe('resolveYamlContext', () => {
  it('suggests key context after recipe newline indent', () => {
    const lines = ['steps:', '  - recipe:', '    '];
    const ctx = resolveYamlContext(doc(lines), 2, 4);
    expect(ctx.parentWire).toBe('recipe');
    expect(ctx.positionKind).toBe('key');
  });

  it('finds edit parent for nested fields', () => {
    const lines = ['steps:', '  - edit:', '    path: lib/a.dart', '    '];
    const ctx = resolveYamlContext(doc(lines), 3, 4);
    expect(ctx.parentWire).toBe('edit');
    expect(ctx.positionKind).toBe('key');
    expect(ctx.siblingKeys).toContain('path');
  });

  it('marks value position on same-line scalar', () => {
    const lines = ["  language: dar"];
    const ctx = resolveYamlContext(doc(lines), 0, lines[0].length);
    expect(ctx.positionKind).toBe('value');
  });
});
