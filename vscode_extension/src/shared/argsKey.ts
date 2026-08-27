import type { ArgValue, ArgValues } from './types';

/** Stable fingerprint for recipe args (sorted keys, recursive). */
export function argsKey(args: ArgValues): string {
  return JSON.stringify(canonicalize(args));
}

export function canonicalize(value: ArgValue): unknown {
  if (Array.isArray(value)) {
    return value.map(canonicalize);
  }
  if (value && typeof value === 'object') {
    const out: Record<string, unknown> = {};
    for (const key of Object.keys(value).sort()) {
      out[key] = canonicalize(value[key]);
    }
    return out;
  }
  return value;
}
