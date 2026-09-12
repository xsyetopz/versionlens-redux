import { isQuote } from "./characters.mjs";

const RUST_RAW_STRING_PATTERN = /^(?:br|cr|r)(?<hashes>#{0,255})"/u;

function scanLineComment(state, char) {
  if (char === "\n") {
    state.lineComment = false;
    state.output += char;
  }
  return 0;
}

function scanBlockComment(state, char, next) {
  if (char === "\n") {
    state.output += char;
  }
  if (char === "*" && next === "/") {
    state.blockComment = false;
    return 1;
  }
  return 0;
}

function scanQuote(state, char, previous) {
  state.output += char;
  if (char === state.quote && previous !== "\\") {
    state.quote = "";
  }
  return 0;
}

function rawStringEnd(source, index) {
  const opening = RUST_RAW_STRING_PATTERN.exec(source.slice(index));
  if (!opening) {
    return -1;
  }
  const closing = `"${opening.groups?.hashes ?? ""}`;
  const contentStart = index + opening[0].length;
  const closingIndex = source.indexOf(closing, contentStart);
  return closingIndex < 0
    ? source.length - 1
    : closingIndex + closing.length - 1;
}

function scanSourceCharacter(source, index, state) {
  const char = source[index];
  const next = source[index + 1];
  const previous = source[index - 1];
  if (state.lineComment) {
    return scanLineComment(state, char);
  }
  if (state.blockComment) {
    return scanBlockComment(state, char, next);
  }
  if (state.quote) {
    return scanQuote(state, char, previous);
  }
  if (char === "/" && next === "/") {
    state.lineComment = true;
    return 1;
  }
  if (char === "/" && next === "*") {
    state.blockComment = true;
    return 1;
  }
  const rawEndIndex = rawStringEnd(source, index);
  if (rawEndIndex >= index) {
    state.output += source.slice(index, rawEndIndex + 1);
    return rawEndIndex - index;
  }
  if (isQuote(char)) {
    state.quote = char;
  }
  state.output += char;
  return 0;
}

function stripComments(source) {
  const state = {
    blockComment: false,
    lineComment: false,
    output: "",
    quote: "",
  };
  for (let index = 0; index < source.length; index += 1) {
    index += scanSourceCharacter(source, index, state);
  }
  return state.output;
}

export { stripComments };
