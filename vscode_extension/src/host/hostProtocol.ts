import { HOST_PROTOCOL } from '../constants';
import {
  ApplyResponse,
  HostCommand,
  ListResponse,
  PreviewResponse,
} from '../types';

export { HostCommand, ListResponse, PreviewResponse, ApplyResponse };

export function extractHostResult(output: string): string | undefined {
  const begin = output.indexOf(HOST_PROTOCOL.resultBegin);
  const end = output.indexOf(HOST_PROTOCOL.resultEnd);
  if (begin === -1 || end === -1 || end < begin) {
    return undefined;
  }
  return output.slice(begin + HOST_PROTOCOL.resultBegin.length, end).trim();
}

export function parseHostResponse<T>(output: string): T | undefined {
  const payload = extractHostResult(output);
  if (payload === undefined) return undefined;
  return JSON.parse(payload) as T;
}
