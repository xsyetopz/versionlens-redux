const RUST_CALL_PATTERN =
  /^(?:return\s+)?(?<callee>[A-Za-z_][\w:]*)\s*\([^{};]*\)\??$/u;
const TYPESCRIPT_CALL_PATTERN =
  /^return\s+(?<callee>[A-Za-z_$][\w$.]*)\s*\([^{};]*\)$/u;
const TRAILING_SEMICOLON_PATTERN = /;$/u;
const UPPERCASE_START_PATTERN = /^[A-Z]/u;

function isSingleCallBody(body, callee, matchingIndex) {
  const callStart = body.indexOf(`${callee}(`);
  if (callStart < 0) {
    return false;
  }
  const openIndex = callStart + callee.length;
  const closeIndex = matchingIndex(body, openIndex, "(", ")", {
    singleQuote: false,
  });
  if (closeIndex < 0) {
    return false;
  }
  const tail = body.slice(closeIndex + 1).trim();
  return tail === "" || tail === "?";
}

function findingForFunction(fn, options, operations) {
  const body = operations
    .stripComments(fn.body)
    .trim()
    .replace(TRAILING_SEMICOLON_PATTERN, "")
    .trim();
  let pattern = TYPESCRIPT_CALL_PATTERN;
  if (fn.language === "rust") {
    pattern = RUST_CALL_PATTERN;
  }
  const match = body.match(pattern);
  if (!match || UPPERCASE_START_PATTERN.test(match[1])) {
    return;
  }
  if (!isSingleCallBody(body, match[1], operations.matchingIndex)) {
    return;
  }
  if (options.ignorePublicPassThroughWrappers && fn.isPublic) {
    return;
  }
  return {
    callee: match[1],
    line: fn.startLine,
    name: fn.name,
    path: fn.path,
  };
}

export function collectWrappers(functions, options, operations) {
  return functions
    .filter(
      (fn) =>
        !(
          options.ignoreTestFilesForPassThrough &&
          (operations.isTestPath(fn.path) || fn.isTestOnly)
        ),
    )
    .map((fn) => findingForFunction(fn, options, operations))
    .filter(Boolean);
}
