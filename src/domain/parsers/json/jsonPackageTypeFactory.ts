import {
  type PackageGitDescriptor,
  type PackageNameDescriptor,
  type PackagePathDescriptor,
  type PackageVersionDescriptor,
  PackageDescriptor,
  createPackageGitDescType,
  createPackageNameDesc,
  createPackageParentDescType,
  createPackagePathDescType,
  createPackageVersionDesc,
  createProjectVersionTypeDesc
} from '#domain/parsers';
import * as JsonC from 'jsonc-parser';

/**
 * Creates a package name descriptor from a JSON key node.
 * @param keyNode The JSON node representing the key.
 * @returns A package name descriptor.
 */
export function createNameDescFromJsonNode(keyNode: JsonC.Node): PackageNameDescriptor {
  const name = keyNode.value;

  const nameRange = {
    start: keyNode.offset,
    end: keyNode.offset,
  };

  return createPackageNameDesc(name, nameRange);
}

/**
 * Creates a package version descriptor from a JSON value node.
 * @param valueNode The JSON node representing the version value.
 * @returns A package version descriptor.
 */
export function createVersionDescFromJsonNode(valueNode: JsonC.Node): PackageVersionDescriptor {
  // +1 and -1 to be inside quotes
  const versionRange = {
    start: valueNode.offset + 1,
    end: valueNode.offset + valueNode.length - 1,
  };

  const { value: version } = valueNode;

  return createPackageVersionDesc(version, versionRange);
}

/**
 * Creates a package path descriptor from a JSON value node.
 * @param valueNode The JSON node representing the path value.
 * @returns A package path descriptor.
 */
export function createPathDescFromJsonNode(valueNode: JsonC.Node): PackagePathDescriptor {
  // +1 and -1 to be inside quotes
  const pathRange = {
    start: valueNode.offset + 1,
    end: valueNode.offset + valueNode.length - 1,
  };

  return createPackagePathDescType(valueNode.value, pathRange);
}

/**
 * Creates a Git descriptor from a JSON value node.
 * @param valueNode The JSON node representing the repository URL.
 * @returns A package Git descriptor.
 */
export function createRepoDescFromJsonNode(valueNode: JsonC.Node): PackageGitDescriptor {
  return createPackageGitDescType(valueNode.value);
}

/**
 * Creates a package descriptor for the project's own version from a JSON node.
 * @param path The path to the project file.
 * @param node The JSON node.
 * @returns A package descriptor for the project version.
 */
export function createProjectVersionDesc(path: string, node: JsonC.Node): PackageDescriptor {
  const nameDesc = createNameDescFromJsonNode(node);
  const versionDesc = createVersionDescFromJsonNode(node);
  const parentDesc = createPackageParentDescType(path);
  const projectVersionDesc = createProjectVersionTypeDesc();
  return new PackageDescriptor([
    nameDesc,
    versionDesc,
    parentDesc,
    projectVersionDesc
  ]);
}