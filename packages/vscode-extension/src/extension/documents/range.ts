import * as vscode from "vscode";
import type { NativeRange } from "../native/output.ts";

export function toRange(range: NativeRange) {
	return new vscode.Range(
		range.start.line,
		range.start.character,
		range.end.line,
		range.end.character,
	);
}
