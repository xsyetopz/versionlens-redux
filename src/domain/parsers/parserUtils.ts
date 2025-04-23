import type { PackageTextRange } from '#domain/parsers';

export function createTextRange(start: number, end: number = start): PackageTextRange {
  return { start, end };
}