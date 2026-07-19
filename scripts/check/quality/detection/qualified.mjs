const CRATE_TYPE_PATTERN = /\bcrate::(?<name>[A-Z][A-Za-z0-9_]*)\b/gu;
const CRATE_CALL_PATTERN =
  /\bcrate::(?<name>[a-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+)\s*\(/gu;
const STDLIB_CALL_PATTERN =
  /\bstd::(?<name>[a-z_][A-Za-z0-9_]*(?:::[A-Za-z_][A-Za-z0-9_]*)+)\s*\(/gu;
const USE_PATTERN = /^(?:pub(?:\([^)]*\))?\s+)?use\s/u;
const TRAILING_OPEN_PAREN_PATTERN = /\($/u;

function recordMatches({
  findings,
  filePath,
  line,
  text,
  pattern,
  kind,
  suggestionForMatch,
}) {
  pattern.lastIndex = 0;
  for (const match of text.matchAll(pattern)) {
    findings.push({
      kind,
      line,
      path: filePath,
      qualified: match[0].trim().replace(TRAILING_OPEN_PAREN_PATTERN, ""),
      suggested: suggestionForMatch(match),
    });
  }
}

function collectLineFindings(findings, filePath, line, lineNumber) {
  recordMatches({
    findings,
    filePath,
    line: lineNumber,
    text: line,
    pattern: CRATE_TYPE_PATTERN,
    kind: "crate-type",
    suggestionForMatch: (match) => match[1],
  });
  recordMatches({
    findings,
    filePath,
    line: lineNumber,
    text: line,
    pattern: CRATE_CALL_PATTERN,
    kind: "crate-module-call",
    suggestionForMatch: (match) =>
      `${match[1].split("::").at(0)}::${match[1].split("::").at(-1)}()`,
  });
  recordMatches({
    findings,
    filePath,
    line: lineNumber,
    text: line,
    pattern: STDLIB_CALL_PATTERN,
    kind: "std-module-call",
    suggestionForMatch: (match) => `${match[1]}()`,
  });
}

export function collectQualifiedPaths(
  files,
  options,
  { isTestPath, stripComments },
) {
  const findings = [];
  for (const file of files) {
    const eligible =
      file.language === "rust" &&
      !(options.ignoreTestFilesForQualifiedPaths && isTestPath(file.path));
    if (eligible) {
      const lines = stripComments(file.source).split("\n");
      for (let index = 0; index < lines.length; index += 1) {
        const line = lines[index];
        if (!USE_PATTERN.test(line.trimStart())) {
          collectLineFindings(findings, file.path, line, index + 1);
        }
      }
    }
  }
  return findings;
}
