import {
  matchingIndex as findMatchingIndex,
  splitTopLevel as splitSourceTopLevel,
} from "./syntax/balanced.mjs";
import { stripComments as removeComments } from "./syntax/comments.mjs";
import {
  normalizeType as normalizeTypeText,
  parseParameters as parseFunctionParameters,
  returnTypeAfter as parseReturnTypeAfter,
} from "./syntax/signatures.mjs";

function lineNumber(source, index) {
  return source.slice(0, index).split("\n").length;
}

function hasCfgTestAttribute(source, index) {
  const lines = source.slice(0, index).split("\n");
  let lineIndex = lines.length - 1;
  while (lineIndex >= 0) {
    const line = lines[lineIndex].trim();
    if (line && !line.startsWith("#[")) {
      return false;
    }
    if (line.includes("cfg(test)")) {
      return true;
    }
    lineIndex -= 1;
  }
  return false;
}

function functionMatchIsPublic(matchText, language) {
  const trimmed = matchText.trimStart();
  if (language === "rust") {
    return (
      trimmed.startsWith("pub ") ||
      trimmed.startsWith("pub(") ||
      trimmed.startsWith("pub(crate)") ||
      trimmed.startsWith("pub(super)")
    );
  }
  return trimmed.startsWith("export ");
}

const matchingIndex = findMatchingIndex;
const normalizeType = normalizeTypeText;
const parseParameters = parseFunctionParameters;
const returnTypeAfter = parseReturnTypeAfter;
const splitTopLevel = splitSourceTopLevel;
const stripComments = removeComments;

export {
  functionMatchIsPublic,
  hasCfgTestAttribute,
  lineNumber,
  matchingIndex,
  normalizeType,
  parseParameters,
  returnTypeAfter,
  splitTopLevel,
  stripComments,
};
