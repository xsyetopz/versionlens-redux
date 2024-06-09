import { createProjectVersionDesc } from 'domain/packages';
import * as JsonC from 'jsonc-parser';
import { createPackageManagerDesc } from './npmPackageTypeFactory';

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