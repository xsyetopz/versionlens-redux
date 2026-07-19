import {
  lineNumber,
  matchingIndex,
  splitTopLevel,
  stripComments,
} from "./syntax.mjs";

const RUST_STRUCT_PATTERN =
  /(?:^|\n)\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+(?<name>[A-Za-z_][A-Za-z0-9_]*)\s*\{/gu;
const TYPESCRIPT_INTERFACE_PATTERN =
  /(?:^|\n)\s*(?:export\s+)?interface\s+(?<name>[A-Za-z_$][\w$]*)[^{}]*\{/gu;
const TYPESCRIPT_TYPE_OBJECT_PATTERN =
  /(?:^|\n)\s*(?:export\s+)?type\s+(?<name>[A-Za-z_$][\w$]*)\s*=\s*\{/gu;
const RUST_FIELD_PATTERN =
  /^(?:pub(?:\([^)]*\))?\s+)?[A-Za-z_][A-Za-z0-9_]*\s*:/u;
const TYPESCRIPT_FIELD_PATTERN = /^(?:readonly\s+)?[A-Za-z_$][\w$]*\??\s*:/u;

function shapePatterns(language) {
  if (language === "rust") {
    return [RUST_STRUCT_PATTERN];
  }
  return [TYPESCRIPT_INTERFACE_PATTERN, TYPESCRIPT_TYPE_OBJECT_PATTERN];
}

function fieldPattern(language) {
  if (language === "rust") {
    return RUST_FIELD_PATTERN;
  }
  return TYPESCRIPT_FIELD_PATTERN;
}

function shapeFields(file, body) {
  let separator = ";";
  if (file.language === "rust") {
    separator = ",";
  }
  return splitTopLevel(stripComments(body), separator, {
    singleQuote: file.language !== "rust",
  })
    .map((field) => field.trim())
    .filter((field) => field && fieldPattern(file.language).test(field));
}

function shapeForMatch(file, match) {
  const bodyOpenIndex = file.source.indexOf("{", match.index);
  const bodyCloseIndex = matchingIndex(file.source, bodyOpenIndex, "{", "}", {
    singleQuote: file.language !== "rust",
  });
  if (bodyOpenIndex < 0 || bodyCloseIndex < 0) {
    return;
  }
  const fields = shapeFields(
    file,
    file.source.slice(bodyOpenIndex + 1, bodyCloseIndex),
  );
  return {
    endLine: lineNumber(file.source, bodyCloseIndex),
    fieldCount: fields.length,
    language: file.language,
    name: match.groups?.name,
    path: file.path,
    startLine: lineNumber(file.source, match.index),
  };
}

function extractObjectShapes(file) {
  const shapes = [];
  for (const pattern of shapePatterns(file.language)) {
    pattern.lastIndex = 0;
    for (const match of file.source.matchAll(pattern)) {
      const shape = shapeForMatch(file, match);
      if (shape) {
        shapes.push(shape);
      }
    }
  }
  return shapes;
}

export { extractObjectShapes };
