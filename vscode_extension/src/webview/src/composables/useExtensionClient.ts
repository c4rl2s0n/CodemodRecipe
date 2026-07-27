import { inject, type InjectionKey } from 'vue';
import type { ExtensionClient } from '../extensionClient';

export const extensionClientKey: InjectionKey<ExtensionClient> =
  Symbol('extensionClient');

export function useExtensionClient(): ExtensionClient {
  const client = inject(extensionClientKey);
  if (!client) {
    throw new Error(
      'useExtensionClient() called without provide(extensionClientKey)'
    );
  }
  return client;
}
