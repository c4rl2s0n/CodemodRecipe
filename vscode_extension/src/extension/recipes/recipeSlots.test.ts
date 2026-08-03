import { describe, expect, it } from 'vitest';
import {
  parseSlotConfig,
  resolveSlot,
  slotsByRecipeId,
  hintChordsForSlot,
} from './recipeSlots';

describe('parseSlotConfig', () => {
  it('parses string and structured forms', () => {
    expect(parseSlotConfig('  a.b  ')).toBe('a.b');
    expect(parseSlotConfig({ recipeId: 'x', mode: 'open', args: { k: 'v' } })).toEqual({
      recipeId: 'x',
      mode: 'open',
      args: { k: 'v' },
    });
    expect(parseSlotConfig({})).toBeUndefined();
  });
});

describe('resolveSlot', () => {
  it('defaults mode and args for string slots', () => {
    expect(resolveSlot('b', 'flutter.add_bloc')).toEqual({
      slot: 'b',
      recipeId: 'flutter.add_bloc',
      mode: 'auto',
      args: {},
    });
  });
});

describe('slotsByRecipeId', () => {
  it('groups and sorts slot ids', () => {
    expect(
      slotsByRecipeId({
        b: 'r1',
        a: { recipeId: 'r1' },
        z: 'r2',
      })
    ).toEqual({ r1: ['a', 'b'], r2: ['z'] });
  });
});

describe('hintChordsForSlot', () => {
  it('documents default prefixes', () => {
    expect(hintChordsForSlot('b')).toEqual({
      run: 'Ctrl+Shift+I b',
      open: 'Ctrl+Shift+T b',
    });
  });
});
