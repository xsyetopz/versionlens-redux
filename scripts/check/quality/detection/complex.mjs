import { normalizeType } from "./syntax.mjs";

const COMPLEX_TYPE_PATTERN =
  /(?:[A-Za-z_][\w:.$]*\s*(?:<[^;{}()]+>|\[[^\]]+\])|(?:Vec|Option|Result|HashMap|BTreeMap|Array|Promise|Record|ReadonlyArray|Map|Set)\s*<[^;{}()]+>)/u;
const RUST_CONCRETE_TYPE_PATTERN =
  /(?:^|\n)\s*(?:pub(?:\([^)]*\))?\s+)?(?:struct|enum)\s+(?<name>[A-Za-z_][A-Za-z0-9_]*)(?:\s*<[^>{}]+>)?\s*[;{]/gu;
const TYPESCRIPT_CONCRETE_TYPE_PATTERN =
  /(?:^|\n)\s*(?:export\s+)?(?:interface|class)\s+(?<name>[A-Za-z_$][\w$]*)/gu;
const COMMON_COMPLEX_TYPE_PATTERN =
  /^(?:Option<(?:String|&str|usize|bool)>|Vec<(?:String|&str)>|Array<string>|Promise<void>)$/u;
const SIMPLE_OPTION_TYPE_PATTERN =
  /^Option<&?(?:lifetime |'static )?(?:str|Self|[A-Za-z_][\w:.$]*(?:<lifetime>)?)>$/u;
const SIMPLE_RESULT_TYPE_PATTERN =
  /^Result<[A-Za-z_][\w:.$]*(?:<lifetime>)?,[A-Za-z_][\w:.$]*(?:<lifetime>)?>$/u;
const TYPE_ALIAS_PATTERN =
  /(?:^|\n)\s*(?:(?:export|pub(?:\([^)]*\))?)\s+)?type\s+(?<name>[A-Za-z_][\w]*)\s*(?:<[^=]+>)?\s*=/gu;
const TYPE_REFERENCE_PREFIX_PATTERN = /^&(?:mut\s+)?/u;
const TYPE_GENERIC_SUFFIX_PATTERN = /<.*$/u;
const TYPE_ARRAY_SUFFIX_PATTERN = /\[.*$/u;

function collectTypeNames(files, patternForFile) {
  const byPath = new Map();
  for (const file of files) {
    const names = new Set();
    const pattern = patternForFile(file);
    pattern.lastIndex = 0;
    for (const match of file.source.matchAll(pattern)) {
      const name = match.groups?.name;
      if (name) {
        names.add(name);
      }
    }
    byPath.set(file.path, names);
  }
  return byPath;
}

function addGlobalNames(files, byPath) {
  const allNames = new Set([...byPath.values()].flatMap((names) => [...names]));
  for (const file of files) {
    const localNames = byPath.get(file.path) ?? [];
    byPath.set(file.path, new Set([...localNames, ...allNames]));
  }
  return byPath;
}

function collectAliasedTypeNames(files) {
  return addGlobalNames(
    files,
    collectTypeNames(files, () => TYPE_ALIAS_PATTERN),
  );
}

function concretePattern(file) {
  if (file.language === "rust") {
    return RUST_CONCRETE_TYPE_PATTERN;
  }
  return TYPESCRIPT_CONCRETE_TYPE_PATTERN;
}

function collectConcreteTypeNames(files) {
  return addGlobalNames(files, collectTypeNames(files, concretePattern));
}

function typeBaseName(typeText) {
  return typeText
    .replace(TYPE_REFERENCE_PREFIX_PATTERN, "")
    .replace(TYPE_GENERIC_SUFFIX_PATTERN, "")
    .replace(TYPE_ARRAY_SUFFIX_PATTERN, "");
}

function isNamedTypeReference(typeText, location, namesByPath) {
  return namesByPath.get(location.path)?.has(typeBaseName(typeText)) ?? false;
}

function isDirectNamedTypeReference(typeText, location, namesByPath) {
  if (!isNamedTypeReference(typeText, location, namesByPath)) {
    return false;
  }
  const normalized = normalizeType(typeText);
  const base = typeBaseName(normalized);
  return (
    normalized === base ||
    normalized.startsWith(`${base}<`) ||
    normalized.startsWith(`&${base}<`) ||
    normalized.startsWith(`&mut ${base}<`)
  );
}

function ignoredType(context) {
  const { aliasesByPath, concreteTypesByPath, location, normalized, options } =
    context;
  if (isNamedTypeReference(normalized, location, aliasesByPath)) {
    return true;
  }
  if (isDirectNamedTypeReference(normalized, location, concreteTypesByPath)) {
    return true;
  }
  if (
    options.ignoreCommonComplexTypes &&
    (COMMON_COMPLEX_TYPE_PATTERN.test(normalized) ||
      SIMPLE_OPTION_TYPE_PATTERN.test(normalized) ||
      SIMPLE_RESULT_TYPE_PATTERN.test(normalized))
  ) {
    return true;
  }
  return options.ignorePublicApiTypes && location.isPublic;
}

function typeKey(normalized, location, options) {
  if (options.complexTypeScope === "file") {
    return `${location.path}:${normalized}`;
  }
  return normalized;
}

function recordType(context) {
  const { byType, location, options, typeText } = context;
  if (!(typeText && COMPLEX_TYPE_PATTERN.test(typeText))) {
    return;
  }
  const normalized = normalizeType(typeText);
  if (ignoredType({ ...context, normalized })) {
    return;
  }
  const key = typeKey(normalized, location, options);
  const locations = byType.get(key) ?? [];
  locations.push(location);
  byType.set(key, locations);
}

function parameterLocation(fn, parameter) {
  return {
    isPublic: fn.isPublic,
    line: fn.startLine,
    owner: fn.name,
    path: fn.path,
    role: `parameter ${parameter.name}`,
  };
}

function returnLocation(fn) {
  return {
    isPublic: fn.isPublic,
    line: fn.startLine,
    owner: fn.name,
    path: fn.path,
    role: "return",
  };
}

function displayedType(key, options) {
  if (options.complexTypeScope === "file") {
    return key.slice(key.indexOf(":") + 1);
  }
  return key;
}

function collectComplexTypes(functions, files, options = {}) {
  const byType = new Map();
  const aliasesByPath = collectAliasedTypeNames(files);
  const concreteTypesByPath = collectConcreteTypeNames(files);
  const shared = { aliasesByPath, byType, concreteTypesByPath, options };
  for (const fn of functions) {
    for (const parameter of fn.parameters) {
      recordType({
        ...shared,
        location: parameterLocation(fn, parameter),
        typeText: parameter.typeText,
      });
    }
    recordType({
      ...shared,
      location: returnLocation(fn),
      typeText: fn.returnType,
    });
  }

  return Array.from(byType.entries())
    .filter(([, locations]) => locations.length > 1)
    .map(([key, locations]) => ({
      count: locations.length,
      locations,
      typeText: displayedType(key, options),
    }))
    .sort(
      (first, second) =>
        second.count - first.count ||
        first.typeText.localeCompare(second.typeText),
    );
}

export { collectComplexTypes };
