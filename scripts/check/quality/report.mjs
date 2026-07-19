import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

function runDiff({
  firstLabel,
  firstSource,
  secondLabel,
  secondSource,
  command,
}) {
  const directory = fs.mkdtempSync(
    path.join(os.tmpdir(), "versionlens-quality-"),
  );
  const firstPath = path.join(directory, "first.txt");
  const secondPath = path.join(directory, "second.txt");
  fs.writeFileSync(firstPath, firstSource);
  fs.writeFileSync(secondPath, secondSource);
  const result = spawnSync(command, ["-u", firstPath, secondPath], {
    encoding: "utf8",
  });
  fs.rmSync(directory, { force: true, recursive: true });

  if (result.error) {
    return `[diff failed: ${result.error.message}]`;
  }
  return result.stdout
    .replaceAll(firstPath, firstLabel)
    .replaceAll(secondPath, secondLabel)
    .trimEnd();
}

function formatDuplicates(lines, findings, command) {
  if (findings.length === 0) {
    return;
  }
  lines.push("duplicate logic");
  for (const finding of findings) {
    const firstLabel = `${finding.firstPath}:${finding.firstStartLine}-${finding.firstEndLine} ${finding.firstName}`;
    const secondLabel = `${finding.secondPath}:${finding.secondStartLine}-${finding.secondEndLine} ${finding.secondName}`;
    lines.push(`- ${firstLabel}`);
    lines.push(`  ${secondLabel}`);
    lines.push(`  similarity=${finding.similarity.toFixed(2)}`);
    lines.push(
      runDiff({
        command,
        firstLabel,
        firstSource: finding.firstSource,
        secondLabel,
        secondSource: finding.secondSource,
      }),
    );
  }
}

function formatComplexTypes(lines, findings) {
  if (findings.length === 0) {
    return;
  }
  lines.push("repeated complex types");
  for (const finding of findings) {
    lines.push(`- ${finding.typeText} count=${finding.count}`);
    for (const location of finding.locations) {
      lines.push(
        `  ${location.path}:${location.line} ${location.owner} ${location.role}`,
      );
    }
  }
}

function formatOversized(lines, shapes, functions) {
  if (shapes.length === 0 && functions.length === 0) {
    return;
  }
  lines.push("oversized shapes");
  for (const shape of shapes) {
    lines.push(
      `- ${shape.path}:${shape.startLine}-${shape.endLine} ${shape.name} fields=${shape.fieldCount}`,
    );
  }
  for (const fn of functions) {
    lines.push(
      `- ${fn.path}:${fn.line} ${fn.name} parameters=${fn.parameterCount}`,
    );
  }
}

function formatParameters(lines, heading, findings) {
  if (findings.length === 0) {
    return;
  }
  lines.push(heading);
  for (const finding of findings) {
    lines.push(
      `- ${finding.path}:${finding.line} ${finding.functionName} parameter=${finding.parameterName}`,
    );
  }
}

function formatWrappers(lines, findings) {
  if (findings.length === 0) {
    return;
  }
  lines.push("pass-through wrappers");
  for (const finding of findings) {
    lines.push(
      `- ${finding.path}:${finding.line} ${finding.name} -> ${finding.callee}`,
    );
  }
}

function formatQualifiedPaths(lines, findings) {
  if (findings.length === 0) {
    return;
  }
  lines.push("overqualified paths");
  for (const finding of findings) {
    lines.push(
      `- ${finding.path}:${finding.line} ${finding.kind} ${finding.qualified} -> ${finding.suggested}`,
    );
  }
}

export function formatFindings(result, options = {}) {
  const lines = [];
  formatDuplicates(
    lines,
    result.duplicateLogic,
    options.diffCommand ?? "difft",
  );
  formatComplexTypes(lines, result.repeatedComplexTypes);
  formatOversized(lines, result.oversizedShapes, result.oversizedFunctions);
  formatParameters(lines, "unused parameters", result.unusedParameters);
  formatParameters(lines, "suppressed parameters", result.suppressedParameters);
  formatWrappers(lines, result.passThroughWrappers);
  formatQualifiedPaths(lines, result.overqualifiedPaths);
  return lines.join("\n");
}

export function formatExceptions(exceptions) {
  if (exceptions.length === 0) {
    return "";
  }
  return [
    `checked quality exceptions (${exceptions.length})`,
    ...exceptions.map(
      (exception) =>
        `- ${exception.category}:${exception.fingerprint} (${exception.owner})`,
    ),
  ].join("\n");
}
