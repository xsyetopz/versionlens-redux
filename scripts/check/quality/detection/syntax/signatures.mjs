import { splitTopLevel } from "./balanced.mjs";

const TYPE_NORMALIZE_PATTERN = /\s+/gu;
const LIFETIME_PATTERN = /'[A-Za-z_][A-Za-z0-9_]*/gu;
const TYPE_SEPARATOR_PATTERN = /\s*(?<separator>[<>,[\]|&])\s*/gu;
const RUST_PARAMETER_PATTERN =
  /^(?:mut\s+)?(?<name>[A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?<type>.+)$/u;
const TYPESCRIPT_TYPED_PARAMETER_PATTERN =
  /^(?:\.\.\.)?(?<name>[A-Za-z_$][\w$]*)\??\s*:\s*(?<type>.+)$/u;
const TYPESCRIPT_DEFAULT_PARAMETER_PATTERN =
  /^(?:\.\.\.)?(?<name>[A-Za-z_$][\w$]*)\??\s*(?:=.*)?$/u;
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
  const match = parameter.match(RUST_PARAMETER_PATTERN);
  if (!match?.groups) {
    return emptyParameter(parameter);
  }
  return {
    name: match.groups.name,
    typeText: normalizeType(match.groups.type),
  };
}

function parseTypescriptParameter(parameter) {
  const typedMatch = parameter.match(TYPESCRIPT_TYPED_PARAMETER_PATTERN);
  if (typedMatch?.groups) {
    return {
      name: typedMatch.groups.name,
      typeText: normalizeType(typedMatch.groups.type),
    };
  }
  const defaultMatch = parameter.match(TYPESCRIPT_DEFAULT_PARAMETER_PATTERN);
  if (defaultMatch?.groups) {
    return emptyParameter(defaultMatch.groups.name);
  }
  return emptyParameter(parameter);
}

function parameterParser(language) {
  if (language === "rust") {
    return parseRustParameter;
  }
  return parseTypescriptParameter;
}

function parseParameters(parameterText, language) {
  let splittableText = parameterText;
  if (language === "rust") {
    splittableText = parameterText.replaceAll(LIFETIME_PATTERN, "lifetime");
  }
  return splitTopLevel(splittableText, ",", {
    singleQuote: language !== "rust",
  })
    .map((parameter) => parameter.trim())
    .filter(Boolean)
    .map(parameterParser(language));
}

function returnTypePattern(language) {
  if (language === "rust") {
    return RUST_RETURN_TYPE_PATTERN;
  }
  return TYPESCRIPT_RETURN_TYPE_PATTERN;
}

function returnTypeAfter(source, parameterCloseIndex, language) {
  const between = source.slice(
    parameterCloseIndex + 1,
    source.indexOf("{", parameterCloseIndex),
  );
  const match = between.match(returnTypePattern(language));
  if (match?.groups) {
    return normalizeType(match.groups.type);
  }
  return "";
}

export { normalizeType, parseParameters, returnTypeAfter };
