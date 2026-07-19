const SIMPLE_CHAR_LITERAL_OFFSET = 2;
const ESCAPED_CHAR_LITERAL_OFFSET = 3;

function isQuote(char, singleQuote = true) {
  return char === '"' || (singleQuote && char === "'") || char === "`";
}

function rustCharLiteralEnd(source, startIndex) {
  if (source[startIndex] !== "'") {
    return -1;
  }
  let offset = SIMPLE_CHAR_LITERAL_OFFSET;
  if (source[startIndex + 1] === "\\") {
    offset = ESCAPED_CHAR_LITERAL_OFFSET;
  }
  if (source[startIndex + offset] === "'") {
    return startIndex + offset;
  }
  return -1;
}

export { isQuote, rustCharLiteralEnd };
