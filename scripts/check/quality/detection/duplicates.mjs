const IDENTIFIER_PATTERN = /[A-Za-z_$][\w$]*/gu;
const STRING_PATTERN = /"(?:\\.|[^"])*"|'(?:\\.|[^'])*'|`(?:\\.|[^`])*`/gu;
const NUMBER_PATTERN = /\b\d+(?:\.\d+)?\b/gu;
const TOKEN_SPLIT_PATTERN = /[^A-Z]+/u;
const DEFAULT_MINIMUM_TOKENS = 3;

function normalizeBody(body) {
  return body
    .replaceAll(STRING_PATTERN, "STR")
    .replaceAll(NUMBER_PATTERN, "NUM")
    .replaceAll(IDENTIFIER_PATTERN, "ID")
    .replaceAll(/\s+/gu, "")
    .trim();
}

function tokenCount(normalizedBody) {
  return normalizedBody.split(TOKEN_SPLIT_PATTERN).filter(Boolean).length;
}

function appendPairs(duplicates, matches) {
  for (let firstIndex = 0; firstIndex < matches.length; firstIndex += 1) {
    for (
      let secondIndex = firstIndex + 1;
      secondIndex < matches.length;
      secondIndex += 1
    ) {
      const first = matches[firstIndex];
      const second = matches[secondIndex];
      duplicates.push({
        firstEndLine: first.endLine,
        firstName: first.name,
        firstPath: first.path,
        firstSource: first.source,
        firstStartLine: first.startLine,
        secondEndLine: second.endLine,
        secondName: second.name,
        secondPath: second.path,
        secondSource: second.source,
        secondStartLine: second.startLine,
        similarity: 1,
      });
    }
  }
}

export function collectDuplicates(functions, options, operations) {
  const byBody = new Map();
  const minimumTokens = options.duplicateMinTokens ?? DEFAULT_MINIMUM_TOKENS;
  for (const fn of functions) {
    const ignoredTestFunction =
      options.ignoreTestFilesForDuplicates &&
      (operations.isTestPath(fn.path) || fn.isTestOnly);
    if (!ignoredTestFunction) {
      const normalizedBody = normalizeBody(operations.stripComments(fn.body));
      if (tokenCount(normalizedBody) >= minimumTokens) {
        const matches = byBody.get(normalizedBody) ?? [];
        matches.push(fn);
        byBody.set(normalizedBody, matches);
      }
    }
  }

  const duplicates = [];
  for (const matches of byBody.values()) {
    if (matches.length > 1) {
      appendPairs(duplicates, matches);
    }
  }
  return duplicates;
}
