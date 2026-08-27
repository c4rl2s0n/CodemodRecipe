import type { HostCommand } from './hostProtocol';

/** Preview/apply timeouts must not fall through to unbounded one-shot retries. */
export function shouldRetryOneShotAfterPersistentFailure(
  command: HostCommand['command'],
  errorMessage: string
): boolean {
  const timedOut = errorMessage.includes('Timed out waiting');
  if (timedOut && (command === 'preview' || command === 'apply')) {
    return false;
  }
  return true;
}
