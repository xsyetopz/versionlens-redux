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
  /(?:^|\n)\s*(?:export\s+)?const\s+(?<name>[A-Za-z_$][\w$]*)\s*=\s*(?:async\s*)?\(/gu;
const RUST_TRAIT_IMPL_PATTERN =
  /(?:^|\n)\s*impl(?:\s*<[^{};]*>)?\s+[^{};]*\bfor\b[^{};]*\{/gu;

function functionPatterns(language) {
  if (language === "rust") {
    return [{ kind: "declaration", pattern: RUST_FUNCTION_PATTERN }];
  }
  return [
    { kind: "declaration", pattern: TYPESCRIPT_FUNCTION_PATTERN },
    { kind: "arrow", pattern: TYPESCRIPT_ARROW_PATTERN },
  ];
}

function quoteOptions(language) {
  return { singleQuote: language !== "rust" };
}

function arrowBodyOpen(source, parameterCloseIndex) {
  const arrowIndex = source.indexOf("=>", parameterCloseIndex + 1);
  if (arrowIndex < 0) {
    return -1;
  }
  const signatureTail = source
    .slice(parameterCloseIndex + 1, arrowIndex)
    .trim();
  if (signatureTail !== "" && !signatureTail.startsWith(":")) {
    return -1;
  }
  const afterArrow = source.slice(arrowIndex + 2);
  const bodyOffset = afterArrow.search(/\S/u);
  return bodyOffset >= 0 && afterArrow[bodyOffset] === "{"
    ? arrowIndex + 2 + bodyOffset
    : -1;
}

function declarationBodyOpen(file, parameterCloseIndex) {
  let bodyOpenIndex = file.source.indexOf("{", parameterCloseIndex + 1);
  while (bodyOpenIndex >= 0 && file.language === "typescript") {
    const typeCloseIndex = matchingIndex(
      file.source,
      bodyOpenIndex,
      "{",
      "}",
      quoteOptions(file.language),
    );
    if (typeCloseIndex < 0) {
      return -1;
    }
    const before = file.source.slice(parameterCloseIndex + 1, bodyOpenIndex);
    const after = file.source.slice(typeCloseIndex + 1).trimStart();
    const closesReturnType = before.includes(":") && /^[>{|&?[,]/u.test(after);
    const precedesBody = before.includes(":") && after.startsWith("{");
    if (!(closesReturnType || precedesBody)) {
      break;
    }
    bodyOpenIndex = file.source.indexOf("{", typeCloseIndex + 1);
  }
  return bodyOpenIndex;
}

function isRustTraitMethod(file, functionIndex) {
  if (file.language !== "rust") {
    return false;
  }
  RUST_TRAIT_IMPL_PATTERN.lastIndex = 0;
  for (const match of file.source.matchAll(RUST_TRAIT_IMPL_PATTERN)) {
    const bodyOpenIndex = match.index + match[0].lastIndexOf("{");
    if (bodyOpenIndex >= functionIndex) {
      return false;
    }
    const bodyCloseIndex = matchingIndex(
      file.source,
      bodyOpenIndex,
      "{",
      "}",
      quoteOptions(file.language),
    );
    if (bodyCloseIndex >= functionIndex) {
      return true;
    }
  }
  return false;
}

function functionBounds(file, match, name, kind) {
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

  const bodyOpenIndex =
    kind === "arrow"
      ? arrowBodyOpen(file.source, parameterCloseIndex)
      : declarationBodyOpen(file, parameterCloseIndex);
  const semicolonIndex = file.source.indexOf(";", parameterCloseIndex + 1);
  if (
    file.language === "rust" &&
    semicolonIndex >= 0 &&
    semicolonIndex < bodyOpenIndex
  ) {
    return;
  }
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

function functionForMatch(file, match, kind) {
  const name = match.groups?.name;
  if (!name) {
    return;
  }
  const bounds = functionBounds(file, match, name, kind);
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
    isPublic:
      functionMatchIsPublic(match[0], file.language) ||
      isRustTraitMethod(file, match.index),
    isTestOnly: hasCfgTestAttribute(file.source, match.index),
    language: file.language,
    name,
    parameters: parseParameters(parameterText, file.language),
    path: file.path,
    returnType: returnTypeAfter(
      file.source,
      parameterCloseIndex,
      bodyOpenIndex,
      file.language,
    ),
    source: file.source.slice(match.index, bodyCloseIndex + 1),
    startLine: lineNumber(file.source, match.index),
  };
}

function extractFunctions(file) {
  const functions = [];
  for (const { kind, pattern } of functionPatterns(file.language)) {
    pattern.lastIndex = 0;
    for (const match of file.source.matchAll(pattern)) {
      const fn = functionForMatch(file, match, kind);
      if (fn) {
        functions.push(fn);
      }
    }
  }
  return functions.sort((first, second) => first.startLine - second.startLine);
}

export { extractFunctions };
