import {
  PackageDescriptor,
  TPackageVersionDescriptor,
  createIgnoreChangesDesc,
  createPackageParentDescType,
} from '#domain/packages';
import { createNameDescFromJsonNode, createVersionDescFromJsonNode } from '#domain/parsers';
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

function createPackageManagerVersionFromJsonNode(valueNode: any): TPackageVersionDescriptor {
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