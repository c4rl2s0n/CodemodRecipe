import { describe, expect, it } from 'vitest';
import { shouldRetryOneShotAfterPersistentFailure } from './hostBridgeRetryPolicy';

describe('shouldRetryOneShotAfterPersistentFailure', () => {
  it('fail-closes preview and apply on timeout', () => {
    expect(
      shouldRetryOneShotAfterPersistentFailure(
        'preview',
        'Timed out waiting for preview response (30000ms)'
      )
    ).toBe(false);
    expect(
      shouldRetryOneShotAfterPersistentFailure(
        'apply',
        'Timed out waiting for apply response (30000ms)'
      )
    ).toBe(false);
  });

  it('still retries list/reload on timeout and non-timeout failures', () => {
    expect(
      shouldRetryOneShotAfterPersistentFailure(
        'list',
        'Timed out waiting for list response (30000ms)'
      )
    ).toBe(true);
    expect(
      shouldRetryOneShotAfterPersistentFailure('preview', 'Host crashed')
    ).toBe(true);
  });
});
