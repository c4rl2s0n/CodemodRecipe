import { HOST_PROTOCOL } from '../constants';
import {
  ApplyResponse,
  DescribeResponse,
  DiffResponse,
  HostCommand,
  ListResponse,
  PreviewResponse,
} from '../types';

export {
  HostCommand,
  ListResponse,
  DescribeResponse,
  DiffResponse,
  PreviewResponse,
  ApplyResponse,
};

export function extractHostResult(output: string): string | undefined {
  const begin = output.indexOf(HOST_PROTOCOL.resultBegin);
  const end = output.indexOf(HOST_PROTOCOL.resultEnd);
  if (begin === -1 || end === -1 || end < begin) {
    return undefined;
  }
  return output.slice(begin + HOST_PROTOCOL.resultBegin.length, end).trim();
}

export function extractHostResultFrame(
  output: string
): { payload: string; rest: string } | undefined {
  const begin = output.indexOf(HOST_PROTOCOL.resultBegin);
  if (begin === -1) {
    return undefined;
  }
  const end = output.indexOf(HOST_PROTOCOL.resultEnd, begin);
  if (end === -1 || end < begin) {
    return undefined;
  }
  const payload = output.slice(begin + HOST_PROTOCOL.resultBegin.length, end).trim();
  const rest = output.slice(end + HOST_PROTOCOL.resultEnd.length);
  return { payload, rest };
}

export function parseHostResponse<T>(output: string): T | undefined {
  const payload = extractHostResult(output);
  if (payload === undefined) return undefined;
  return JSON.parse(payload) as T;
}
