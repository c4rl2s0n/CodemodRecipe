import { describe, expect, it } from 'vitest';
import type { TranscriptEvent } from '../shared';
import {
  eventIsExpandable,
  formatExpandedDetail,
  formatStatsBadges,
  looksLikeJson,
} from './transcript';

describe('transcript helpers', () => {
  it('detects JSON-looking detail', () => {
    expect(looksLikeJson('{"a":1}')).toBe(true);
    expect(looksLikeJson('[1,2]')).toBe(true);
    expect(looksLikeJson('hello')).toBe(false);
  });

  it('formats stats badges only for positive counts', () => {
    const event: TranscriptEvent = {
      kind: 'replace',
      detail: 'a.dart',
      stats: { addedLines: 2, removedLines: 1 },
    };
    expect(formatStatsBadges(event)).toEqual(['+2', '-1']);
    expect(
      formatStatsBadges({
        kind: 'log',
        detail: 'x',
        stats: { addedLines: 0, removedLines: 0 },
      })
    ).toEqual([]);
  });

  it('pretty-prints JSON payload on expand', () => {
    const event: TranscriptEvent = {
      kind: 'log',
      detail: '{"n":1}',
      expandable: true,
      payload: { json: { n: 1, path: 'x' } },
    };
    expect(eventIsExpandable(event)).toBe(true);
    expect(formatExpandedDetail(event)).toContain('"n": 1');
    expect(formatExpandedDetail(event)).toContain('"path": "x"');
  });

  it('falls back to payload object pretty-print', () => {
    const event: TranscriptEvent = {
      kind: 'insert',
      detail: 'lib/a.dart',
      expandable: true,
      payload: { capture: 'root', text: 'x' },
    };
    const formatted = formatExpandedDetail(event);
    expect(formatted).toContain('"capture": "root"');
    expect(formatted).toContain('"text": "x"');
  });
});
