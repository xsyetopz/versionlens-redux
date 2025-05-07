import {
  type PackagePathDescriptor,
  type PackageVersionDescriptor,
  PackageDescriptor,
  createIgnoreChangesDesc,
  createNameDescFromJsonNode,
  createPackageParentDescType,
  createPathDescFromJsonNode,
  createProjectVersionDesc,
  createVersionDescFromJsonNode,
} from '#domain/parsers';
import * as JsonC from 'jsonc-parser';

/**
 * A regex to match the `package.json`'s `packageManager` value.
 *
 * @example packageManager@version
 */
export const packageManagerVersionRegex = /^([\w]+)@(.+)$/;

export function createPackageManagerDesc(path: string, node: JsonC.Node): PackageDescriptor {
  const nameDesc = createNameDescFromJsonNode(node);
  const versionDesc = createPackageManagerVersionFromJsonNode(node);
  const parentDesc = createPackageParentDescType(path);
  const ignoreChangesDesc = createIgnoreChangesDesc();
  return new PackageDescriptor([
    nameDesc,
    versionDesc,
    parentDesc,
    ignoreChangesDesc
  ]);
}

function createPackageManagerVersionFromJsonNode(valueNode: JsonC.Node): PackageVersionDescriptor {
  const versionDesc = createVersionDescFromJsonNode(valueNode);

  // Handle packageManager field
  const [_, packageName, packageVersion] =
    packageManagerVersionRegex.exec(valueNode.value) ?? [];

  if (packageVersion != null) {
    versionDesc.version = packageVersion;
    versionDesc.versionRange.start += packageName.length + 1;
  }

  return versionDesc;
}

export function customDescriptorHandler(
  path: string,
  node: JsonC.Node
): PackageDescriptor | undefined {
  if (node.type !== 'string') return;

  const children = node.parent?.children
  if (!children) return;

  const firstChild = children[0];
  switch (firstChild.value) {
    case 'packageManager':
      return createPackageManagerDesc(path, node);
    case 'version':
      return createProjectVersionDesc(path, node);
  }
}

export function createNpmVersionDescFromJsonNode(
  valueNode: JsonC.Node
): PackagePathDescriptor | PackageVersionDescriptor {
  const { value: version } = valueNode;
  if (version.startsWith('file:'))
    return createPathDescFromJsonNode(valueNode)
  else if (version.startsWith('link:')) {
    (valueNode as any).value = valueNode.value.replace('link:', 'file:')
      + '/package.json';
    return createPathDescFromJsonNode(valueNode)
  }
  return createVersionDescFromJsonNode(valueNode);
}