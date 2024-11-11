import { TPackageTextRange } from '#domain/parsers';

export function createDependencyRange(start: number, end: number): TPackageTextRange {
  return { start, end };
}