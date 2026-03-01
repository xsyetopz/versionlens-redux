import { AST } from "toml-eslint-parser";

/**
 * Checks if a complex TOML node has a property with a specific name.
 * @param node The TOML key-value node.
 * @param type The property name to check for.
 * @returns True if the property exists, otherwise false.
 */
export function complexHasProperty(node: AST.TOMLKeyValue, type: string) {
  const index = node.key.keys.findIndex(x => x.type === 'TOMLBare' && x.name === type);
  return index > -1;
}

/**
 * Checks if a set of keys matches any of the provided table expressions.
 * Supports wildcard '*' in expressions.
 * @param keys The array of keys representing the current location in the TOML tree.
 * @param matchExpressions The array of match expressions.
 * @returns The matching expression string if found, otherwise an empty string.
 */
export function matchesTableExpression(keys: (string | number)[], matchExpressions: string[]) {
  let found = false;
  let foundExpr = "";

  for (let exprIndex = 0; exprIndex < matchExpressions.length; exprIndex++) {
    const expr = matchExpressions[exprIndex];
    const components = expr.split('.');

    if (keys.length != components.length) continue;

    found = true;
    for (let index = 0; index < components.length; index++) {
      if (components[index] === '*') continue;
      if (keys[index] != components[index]) {
        found = false;
        break;
      };
    }

    if (found) {
      foundExpr = expr;
      break;
    }
  }

  return foundExpr;
}