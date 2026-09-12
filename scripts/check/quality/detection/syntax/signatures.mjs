import { splitTopLevel } from "./balanced.mjs";

const TYPE_NORMALIZE_PATTERN = /\s+/gu;
const LIFETIME_PATTERN = /'[A-Za-z_][A-Za-z0-9_]*/gu;
const TYPE_SEPARATOR_PATTERN = /\s*(?<separator>[<>,[\]|&])\s*/gu;
const RUST_PARAMETER_PATTERN =
  /^(?:mut\s+)?(?<name>[A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?<type>.+)$/u;
const TYPESCRIPT_TYPED_PARAMETER_PATTERN =
  /^(?:\.\.\.)?(?<name>[A-Za-z_$][\w$]*)\??\s*:\s*(?<type>[\s\S]+)$/u;
const TYPESCRIPT_DEFAULT_PARAMETER_PATTERN =
  /^(?:\.\.\.)?(?<name>[A-Za-z_$][\w$]*)\??\s*(?:=[\s\S]*)?$/u;
const RUST_RECEIVER_PATTERN =
  /^(?:mut\s+)?self$|^&\s*(?:'[A-Za-z_][A-Za-z0-9_]*\s+)?(?:mut\s+)?self$/u;
const RUST_DESTRUCTURED_PARAMETER_PATTERN =
  /^(?<pattern>[([{][\s\S]*[)\]}])\s*:\s*(?<type>[\s\S]+)$/u;
const RUST_BINDING_PATTERN = /[A-Za-z_][A-Za-z0-9_]*/gu;
const RUST_PATTERN_KEYWORDS = new Set(["mut", "ref", "self"]);
const RUST_RETURN_TYPE_PATTERN = /->\s*(?<type>[^{}]+)$/u;
const TYPESCRIPT_RETURN_TYPE_PATTERN = /^\s*:\s*(?<type>[^={]+)$/u;

function compactWhitespace(source) {
  return source.replaceAll(TYPE_NORMALIZE_PATTERN, " ").trim();
}

function normalizeType(typeText) {
  return compactWhitespace(typeText)
    .replaceAll(LIFETIME_PATTERN, "lifetime")
    .replaceAll(TYPE_SEPARATOR_PATTERN, "$<separator>")
    .replaceAll(TYPE_NORMALIZE_PATTERN, " ");
}

function emptyParameter(name) {
  return { name, typeText: "" };
}

function parseRustParameter(parameter) {
  if (RUST_RECEIVER_PATTERN.test(parameter)) {
    return [emptyParameter("self")];
  }
  const match = parameter.match(RUST_PARAMETER_PATTERN);
  if (match?.groups) {
    return [
      {
        name: match.groups.name,
        typeText: normalizeType(match.groups.type),
      },
    ];
  }
  const destructured = parameter.match(RUST_DESTRUCTURED_PARAMETER_PATTERN);
  if (destructured?.groups) {
    return [...destructured.groups.pattern.matchAll(RUST_BINDING_PATTERN)]
      .map(([name]) => name)
      .filter(
        (name) =>
          !RUST_PATTERN_KEYWORDS.has(name) &&
          name !== "_" &&
          !/^[A-Z]/u.test(name),
      )
      .map(emptyParameter);
  }
  return [emptyParameter(parameter)];
}

function parseTypescriptParameter(parameter) {
  const typedMatch = parameter.match(TYPESCRIPT_TYPED_PARAMETER_PATTERN);
  if (typedMatch?.groups) {
    return [
      {
        name: typedMatch.groups.name,
        typeText: normalizeType(typedMatch.groups.type),
      },
    ];
  }
  const defaultMatch = parameter.match(TYPESCRIPT_DEFAULT_PARAMETER_PATTERN);
  if (defaultMatch?.groups) {
    return [emptyParameter(defaultMatch.groups.name)];
  }
  return [emptyParameter(parameter)];
}

function parameterParser(language) {
  if (language === "rust") {
    return parseRustParameter;
  }
  return parseTypescriptParameter;
}

function parseParameters(parameterText, language) {
  return splitTopLevel(parameterText, ",", {
    singleQuote: language !== "rust",
  })
    .map((parameter) => parameter.trim())
    .filter(Boolean)
    .flatMap(parameterParser(language));
}

function returnTypePattern(language) {
  if (language === "rust") {
    return RUST_RETURN_TYPE_PATTERN;
  }
  return TYPESCRIPT_RETURN_TYPE_PATTERN;
}

function returnTypeAfter(source, parameterCloseIndex, bodyOpenIndex, language) {
  const between = source
    .slice(parameterCloseIndex + 1, bodyOpenIndex)
    .replace(/=>\s*$/u, "")
    .trimEnd();
  const match = between.match(returnTypePattern(language));
  if (match?.groups) {
    return normalizeType(match.groups.type);
  }
  return "";
}

export { normalizeType, parseParameters, returnTypeAfter };
