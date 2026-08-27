/**
 * @vitest-environment happy-dom
 */
import { describe, expect, it, vi } from 'vitest';
import {
  focusFirstControl,
  focusNextInContainer,
  isFocusable,
  listFocusable,
} from './focus';

function mountForm(html: string): HTMLElement {
  document.body.innerHTML = html;
  return document.body.querySelector('[data-recipe-form]') as HTMLElement;
}

describe('focus helpers', () => {
  it('skips tabindex=-1 when listing focusable elements', () => {
    const root = mountForm(`
      <div data-recipe-form>
        <input id="a" />
        <button type="button" tabindex="-1" id="remove">Remove</button>
        <button type="button" id="add">Add</button>
      </div>
    `);
    const ids = listFocusable(root).map((el) => el.id);
    expect(ids).toEqual(['a', 'add']);
    expect(isFocusable(document.getElementById('remove')!)).toBe(false);
  });

  it('focusNextInContainer advances and wraps within root', () => {
    const root = mountForm(`
      <div data-recipe-form>
        <input id="a" />
        <input id="b" />
        <button type="button" id="add">Add</button>
      </div>
    `);
    const a = document.getElementById('a')!;
    a.focus();
    expect(focusNextInContainer(root, a)).toBe(true);
    expect(document.activeElement?.id).toBe('b');
    expect(focusNextInContainer(root)).toBe(true);
    expect(document.activeElement?.id).toBe('add');
    expect(focusNextInContainer(root)).toBe(true);
    expect(document.activeElement?.id).toBe('a');
  });

  it('focusFirstControl prefers input over buttons', () => {
    const root = mountForm(`
      <div data-recipe-form>
        <button type="button" id="summary">Item</button>
        <input id="field" value="x" />
      </div>
    `);
    const focusSpy = vi.spyOn(
      document.getElementById('field') as HTMLInputElement,
      'focus'
    );
    expect(focusFirstControl(root)).toBe(true);
    expect(focusSpy).toHaveBeenCalled();
  });
});
