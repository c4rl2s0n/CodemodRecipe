/** Focus helpers for recipe arg forms (Enter-as-Tab, skip tabindex=-1). */

const FOCUSABLE_SELECTOR =
  'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])';

export function isFocusable(el: Element): el is HTMLElement {
  if (!(el instanceof HTMLElement)) {
    return false;
  }
  if (el.hasAttribute('disabled')) {
    return false;
  }
  const tabIndex = el.getAttribute('tabindex');
  if (tabIndex === '-1') {
    return false;
  }
  if (el.getAttribute('aria-hidden') === 'true') {
    return false;
  }
  // Hidden via display/visibility
  const style = window.getComputedStyle(el);
  if (style.display === 'none' || style.visibility === 'hidden') {
    return false;
  }
  return true;
}

export function listFocusable(root: ParentNode): HTMLElement[] {
  return Array.from(root.querySelectorAll(FOCUSABLE_SELECTOR)).filter(isFocusable);
}

export function focusFirstControl(root: ParentNode | null | undefined): boolean {
  if (!root) {
    return false;
  }
  const first = listFocusable(root).find(
    (el) =>
      el.tagName === 'INPUT' ||
      el.tagName === 'SELECT' ||
      el.tagName === 'TEXTAREA' ||
      el.getAttribute('contenteditable') === 'true'
  );
  if (!first) {
    const any = listFocusable(root)[0];
    if (!any) {
      return false;
    }
    any.focus();
    return true;
  }
  first.focus();
  if (first instanceof HTMLInputElement && first.type === 'text') {
    first.select();
  }
  return true;
}

/**
 * Move focus to the next focusable within `root` after `from` (or document.activeElement).
 * Returns true if focus moved.
 */
export function focusNextInContainer(
  root: ParentNode | null | undefined,
  from?: Element | null
): boolean {
  if (!root) {
    return false;
  }
  const items = listFocusable(root);
  if (items.length === 0) {
    return false;
  }
  const current =
    from instanceof Element
      ? from
      : typeof document !== 'undefined'
        ? document.activeElement
        : null;
  let index = current ? items.indexOf(current as HTMLElement) : -1;
  if (index < 0 && current instanceof Element) {
    // Active element may be nested; find closest focusable ancestor in list
    index = items.findIndex((el) => el.contains(current));
  }
  const next = items[(index + 1) % items.length];
  if (!next) {
    return false;
  }
  next.focus();
  return true;
}

export function onEnterFocusNext(event: KeyboardEvent, root: ParentNode | null | undefined): void {
  if (event.key !== 'Enter' || event.shiftKey || event.ctrlKey || event.metaKey || event.altKey) {
    return;
  }
  // Let textarea keep newlines
  if (event.target instanceof HTMLTextAreaElement) {
    return;
  }
  event.preventDefault();
  focusNextInContainer(root, event.target as Element | null);
}
