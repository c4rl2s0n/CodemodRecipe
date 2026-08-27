import { describe, expect, it } from 'vitest';
import { argsKey } from './argsKey';

describe('argsKey', () => {
  it('is stable across key insertion order', () => {
    const a = argsKey({ b: '2', a: '1' });
    const b = argsKey({ a: '1', b: '2' });
    expect(a).toBe(b);
  });

  it('changes when values change', () => {
    expect(argsKey({ a: '1' })).not.toBe(argsKey({ a: '2' }));
  });
});
