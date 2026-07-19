import { isQuote, rustCharLiteralEnd } from "./characters.mjs";

function quoteEnd(source, index, state, singleQuote) {
  const char = source[index];
  const previous = source[index - 1];
  if (state.quote) {
    if (char === state.quote && previous !== "\\") {
      state.quote = "";
    }
    return index;
  }
  if (!singleQuote && char === "'") {
    const literalEnd = rustCharLiteralEnd(source, index);
    if (literalEnd > index) {
      return literalEnd;
    }
  }
  if (isQuote(char, singleQuote)) {
    state.quote = char;
    return index;
  }
}

function matchingIndex(...parameters) {
  const [source, openIndex, openChar, closeChar, options = {}] = parameters;
  const state = { depth: 0, quote: "" };
  const singleQuote = options.singleQuote !== false;
  for (let index = openIndex; index < source.length; index += 1) {
    const quotedEnd = quoteEnd(source, index, state, singleQuote);
    if (quotedEnd !== undefined) {
      index = quotedEnd;
    } else if (source[index] === openChar) {
      state.depth += 1;
    } else if (source[index] === closeChar) {
      state.depth -= 1;
      if (state.depth === 0) {
        return index;
      }
    }
  }
  return -1;
}

function updateDepths(depths, char) {
  if (char === "<") {
    depths.angle += 1;
  } else if (char === ">" && depths.angle > 0) {
    depths.angle -= 1;
  } else if (char === "{") {
    depths.brace += 1;
  } else if (char === "}" && depths.brace > 0) {
    depths.brace -= 1;
  } else if (char === "[") {
    depths.bracket += 1;
  } else if (char === "]" && depths.bracket > 0) {
    depths.bracket -= 1;
  } else if (char === "(") {
    depths.paren += 1;
  } else if (char === ")" && depths.paren > 0) {
    depths.paren -= 1;
  }
}

function atTopLevel(depths) {
  return (
    depths.angle === 0 &&
    depths.brace === 0 &&
    depths.bracket === 0 &&
    depths.paren === 0
  );
}

function splitTopLevel(source, separator, options = {}) {
  const parts = [];
  const depths = { angle: 0, brace: 0, bracket: 0, paren: 0 };
  const quoteState = { quote: "" };
  const singleQuote = options.singleQuote !== false;
  let start = 0;
  for (let index = 0; index < source.length; index += 1) {
    const quotedEnd = quoteEnd(source, index, quoteState, singleQuote);
    if (quotedEnd === undefined) {
      const char = source[index];
      updateDepths(depths, char);
      if (char === separator && atTopLevel(depths)) {
        parts.push(source.slice(start, index));
        start = index + 1;
      }
    } else {
      index = quotedEnd;
    }
  }
  parts.push(source.slice(start));
  return parts;
}

export { matchingIndex, splitTopLevel };
