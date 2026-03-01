import type { PackageTextRange } from '#domain/parsers';

/**
 * Creates a text range object.
 * @param start The starting offset.
 * @param end The ending offset (defaults to start).
 * @returns A package text range object.
 */
export function createTextRange(start: number, end: number = start): PackageTextRange {
  return { start, end };
}