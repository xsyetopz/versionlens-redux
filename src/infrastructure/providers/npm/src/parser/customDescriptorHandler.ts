import { createProjectVersionDesc } from '#infrastructure/parsers';
import { createPackageManagerDesc } from '#providers/npm';
import * as JsonC from 'jsonc-parser';

export function customDescriptorHandler(path: string, node: JsonC.Node) {
  if (node.type !== 'string') return;

  const parent = node.parent.children[0];

  switch (parent.value) {
    case 'packageManager':
      return createPackageManagerDesc(path, node);
    case 'version':
      return createProjectVersionDesc(path, node);
  }
}