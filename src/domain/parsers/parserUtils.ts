import { TPackageTextRange } from '#domain/parsers';

export function createTextRange(start: number, end: number): TPackageTextRange {
  return { start, end };
}