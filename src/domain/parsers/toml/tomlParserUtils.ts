import { AST } from "toml-eslint-parser";

export function complexHasProperty(node: AST.TOMLKeyValue, type: string) {
  const index = node.key.keys.findIndex(x => x.type === 'TOMLBare' && x.name === type);
  return index > -1;
}

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