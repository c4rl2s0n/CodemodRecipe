import type { TranscriptEvent } from '../shared';

export function eventIsExpandable(event: TranscriptEvent): boolean {
  if (event.expandable) {
    return true;
  }
  if (event.payload && Object.keys(event.payload).length > 0) {
    return true;
  }
  return looksLikeJson(event.detail);
}

export function looksLikeJson(text: string): boolean {
  const trimmed = text.trim();
  return (
    (trimmed.startsWith('{') && trimmed.endsWith('}')) ||
    (trimmed.startsWith('[') && trimmed.endsWith(']'))
  );
}

export function formatExpandedDetail(event: TranscriptEvent): string {
  const payload = event.payload;
  if (payload && typeof payload.json !== 'undefined') {
    try {
      return JSON.stringify(payload.json, null, 2);
    } catch {
      return String(payload.json);
    }
  }
  if (payload && Object.keys(payload).length > 0) {
    const keys = Object.keys(payload);
    if (keys.length === 1 && typeof payload.text === 'string') {
      return payload.text;
    }
    try {
      return JSON.stringify(payload, null, 2);
    } catch {
      return String(payload);
    }
  }
  if (looksLikeJson(event.detail)) {
    try {
      return JSON.stringify(JSON.parse(event.detail), null, 2);
    } catch {
      return event.detail;
    }
  }
  return event.detail;
}

export function formatStatsBadges(event: TranscriptEvent): string[] {
  const badges: string[] = [];
  const added = event.stats?.addedLines;
  const removed = event.stats?.removedLines;
  if (typeof added === 'number' && added > 0) {
    badges.push(`+${added}`);
  }
  if (typeof removed === 'number' && removed > 0) {
    badges.push(`-${removed}`);
  }
  return badges;
}
