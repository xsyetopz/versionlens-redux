import { Range } from "#vscode-host";
import type { NativeRange } from "../native/output.ts";

export function toRange(range: NativeRange): Range {
  return new Range(
    range.start.line,
    range.start.character,
    range.end.line,
    range.end.character,
  );
}
