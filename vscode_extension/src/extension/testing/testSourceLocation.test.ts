import { describe, expect, it } from 'vitest';
import { findTestFunctionLine } from './testSourceLocation';

const SUITE = `return {
  id = "flutter.model.tests",
  tests = {
    parses_fields = function(t)
      local ws = t:workspace({})
    end,

    add_store_field = function(t)
    end,
  },
}
`;

describe('findTestFunctionLine', () => {
  it('returns the 0-based line of the test function', () => {
    expect(findTestFunctionLine(SUITE, 'parses_fields')).toBe(3);
    expect(findTestFunctionLine(SUITE, 'add_store_field')).toBe(7);
  });

  it('falls back to line 0 when the convention is not followed', () => {
    expect(findTestFunctionLine(SUITE, 'missing_test')).toBe(0);
  });

  it('does not match a prefix of a longer test name', () => {
    expect(findTestFunctionLine(SUITE, 'parses')).toBe(0);
  });

  it('escapes regex metacharacters in test names', () => {
    const source = 'return { tests = { ["weird.name"] = function(t) end } }';
    expect(findTestFunctionLine(source, 'weird.name')).toBe(0);
    // And must not throw or false-positive on the bare form:
    expect(findTestFunctionLine(SUITE, 'parses.fields')).toBe(0);
  });
});
