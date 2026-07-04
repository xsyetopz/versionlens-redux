import * as vscode from "vscode";
import { toRange } from "../documents.ts";
import type { NativeDiagnosticPayload } from "../native/output.ts";

export function toDiagnostic(diagnostic: NativeDiagnosticPayload) {
	const rendered = new vscode.Diagnostic(
		toRange(diagnostic.range),
		diagnostic.message,
		diagnostic.severity as vscode.DiagnosticSeverity,
	);
	if (diagnostic.source) {
		rendered.source = diagnostic.source;
	}
	if (diagnostic.code) {
		rendered.code = diagnostic.codeDescriptionUrl
			? {
					target: vscode.Uri.parse(diagnostic.codeDescriptionUrl),
					value: diagnostic.code,
				}
			: diagnostic.code;
	}

	return rendered;
}
