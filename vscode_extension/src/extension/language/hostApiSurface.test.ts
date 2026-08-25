import { describe, expect, it } from 'vitest';
import {
  BUILTIN_INPUT_KINDS,
  HOST_METHODS,
  RECIPE_MODULE_KEYS,
  hostMethodByName,
} from './hostApiSurface';

describe('hostApiSurface', () => {
  it('includes core edit methods with expected keys', () => {
    const insert = hostMethodByName('insert');
    expect(insert?.keys).toEqual(
      expect.arrayContaining(['path', 'query', 'capture', 'text', 'anchor'])
    );
    expect(insert?.enums?.anchor).toEqual(['start', 'end']);

    const run = hostMethodByName('run');
    expect(run).toBeDefined();
    expect(run?.keys).toEqual([]);
  });

  it('lists all build_ctx methods', () => {
    const names = HOST_METHODS.map((m) => m.name);
    expect(names).toEqual(
      expect.arrayContaining([
        'insert',
        'replace',
        'remove',
        'create',
        'delete',
        'query',
        'map',
        'var',
        'queryLib',
        'run',
        'assert',
      ])
    );
    expect(new Set(names).size).toBe(names.length);
  });

  it('exposes recipe module keys and builtin input kinds', () => {
    expect(RECIPE_MODULE_KEYS).toContain('apply');
    expect(RECIPE_MODULE_KEYS).toContain('id');
    expect(BUILTIN_INPUT_KINDS).toContain('file');
    expect(BUILTIN_INPUT_KINDS).toContain('choice');
  });
});
