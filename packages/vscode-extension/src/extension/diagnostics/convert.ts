import { Diagnostic, type DiagnosticSeverity, Uri } from "#vscode-host";
import { toRange } from "../documents/range.ts";
import type { NativeDiagnosticPayload } from "../native/output.ts";

export function toDiagnostic(diagnostic: NativeDiagnosticPayload): Diagnostic {
  const rendered = new Diagnostic(
    toRange(diagnostic.range),
    diagnostic.message,
    diagnostic.severity as DiagnosticSeverity,
  );
  if (diagnostic.source) {
    rendered.source = diagnostic.source;
  }
  if (diagnostic.code) {
    rendered.code = diagnostic.code;
    if (diagnostic.codeDescriptionUrl) {
      rendered.code = {
        target: Uri.parse(diagnostic.codeDescriptionUrl),
        value: diagnostic.code,
      };
    }
  }

  return rendered;
}
