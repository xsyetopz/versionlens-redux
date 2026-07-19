import {
  functionMatchIsPublic,
  hasCfgTestAttribute,
  lineNumber,
  matchingIndex,
  parseParameters,
  returnTypeAfter,
} from "./syntax.mjs";

const RUST_FUNCTION_PATTERN =
  /(?:^|\n)\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(?<name>[A-Za-z_][A-Za-z0-9_]*)\s*\(/gu;
const TYPESCRIPT_FUNCTION_PATTERN =
  /(?:^|\n)\s*(?:export\s+)?(?:async\s+)?function\s+(?<name>[A-Za-z_$][\w$]*)\s*\(/gu;
const TYPESCRIPT_ARROW_PATTERN =
  /(?:^|\n)\s*(?:export\s+)?const\s+(?<name>[A-Za-z_$][\w$]*)\s*=\s*(?:async\s*)?\([^)]*\)\s*=>/gu;

function functionPatterns(language) {
  if (language === "rust") {
    return [RUST_FUNCTION_PATTERN];
  }
  return [TYPESCRIPT_FUNCTION_PATTERN, TYPESCRIPT_ARROW_PATTERN];
}

function quoteOptions(language) {
  return { singleQuote: language !== "rust" };
}

function functionBounds(file, match, name) {
  const nameIndex = match.index + match[0].lastIndexOf(name);
  const parameterOpenIndex = file.source.indexOf("(", nameIndex);
  const parameterCloseIndex = matchingIndex(
    file.source,
    parameterOpenIndex,
    "(",
    ")",
    quoteOptions(file.language),
  );
  if (parameterOpenIndex < 0 || parameterCloseIndex < 0) {
    return;
  }

  const bodyOpenIndex = file.source.indexOf("{", parameterCloseIndex);
  const bodyCloseIndex = matchingIndex(
    file.source,
    bodyOpenIndex,
    "{",
    "}",
    quoteOptions(file.language),
  );
  if (bodyOpenIndex < 0 || bodyCloseIndex < 0) {
    return;
  }
  return {
    bodyCloseIndex,
    bodyOpenIndex,
    parameterCloseIndex,
    parameterOpenIndex,
  };
}

function functionForMatch(file, match) {
  const name = match.groups?.name;
  if (!name) {
    return;
  }
  const bounds = functionBounds(file, match, name);
  if (!bounds) {
    return;
  }
  const {
    bodyCloseIndex,
    bodyOpenIndex,
    parameterCloseIndex,
    parameterOpenIndex,
  } = bounds;
  const parameterText = file.source.slice(
    parameterOpenIndex + 1,
    parameterCloseIndex,
  );

  return {
    body: file.source.slice(bodyOpenIndex + 1, bodyCloseIndex),
    endLine: lineNumber(file.source, bodyCloseIndex),
    isPublic: functionMatchIsPublic(match[0], file.language),
    isTestOnly: hasCfgTestAttribute(file.source, match.index),
    language: file.language,
    name,
    parameters: parseParameters(parameterText, file.language),
    path: file.path,
    returnType: returnTypeAfter(
      file.source,
      parameterCloseIndex,
      file.language,
    ),
    source: file.source.slice(match.index, bodyCloseIndex + 1),
    startLine: lineNumber(file.source, match.index),
  };
}

function extractFunctions(file) {
  const functions = [];
  for (const pattern of functionPatterns(file.language)) {
    pattern.lastIndex = 0;
    for (const match of file.source.matchAll(pattern)) {
      const fn = functionForMatch(file, match);
      if (fn) {
        functions.push(fn);
      }
    }
  }
  return functions.sort((first, second) => first.startLine - second.startLine);
}

export { extractFunctions };
