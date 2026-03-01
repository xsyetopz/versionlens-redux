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

/**
 * Creates a package descriptor for the 'packageManager' property in package.json.
 * @param path The path to the package file.
 * @param node The JSON node representing the packageManager value.
 * @returns A package descriptor.
 */
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

/**
 * Extracts the version from a packageManager string node.
 * @param valueNode The JSON node.
 * @returns A package version descriptor.
 */
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

/**
 * Custom descriptor handler for special package.json properties like 'packageManager' and 'version'.
 * @param path The path to the package file.
 * @param node The JSON node to handle.
 * @returns A package descriptor or undefined.
 */
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

/**
 * Creates a version or path descriptor from a JSON node, handling 'file:' and 'link:' protocols.
 * @param valueNode The JSON node representing the version.
 * @returns A package path or version descriptor.
 */
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