import { createProjectVersionDesc } from '#domain/parsers';
import * as JsonC from 'jsonc-parser';

/**
 * Creates a project version descriptor from a JSON node if it matches the 'version' property.
 * @param path The path to the project file.
 * @param node The JSON node to check.
 * @returns A package version descriptor if the node is a 'version' property, otherwise undefined.
 */
export function createComposerProjectVersionDesc(path: string, node: JsonC.Node) {
  if (node.type !== 'string' || !node.parent || !node.parent.children) return;

  const parent = node.parent.children[0];

  switch (parent.value) {
    case 'version':
      return createProjectVersionDesc(path, node);
  }
}