import {
  type PackageGitDescriptor,
  type PackageNameDescriptor,
  type PackagePathDescriptor,
  type PackageVersionDescriptor,
  PackageDescriptor,
  PackageDescriptorType,
  createPackageGitDescType,
  createPackageNameDesc,
  createPackagePathDescType,
  createPackageVersionDesc,
  createProjectVersionTypeDesc
} from '#domain/parsers';
import { AST } from 'toml-eslint-parser';

/**
 * Gets the map of handlers for complex TOML types.
 * @returns A dictionary of TOML package type handlers.
 */
export function getTomlComplexTypeHandlers() {
  return {
    [PackageDescriptorType.version]: createVersionDescFromTomlNode,
    [PackageDescriptorType.path]: createPathDescFromTomlNode,
    [PackageDescriptorType.git]: createGitDescFromTomlNode
  }
}

/**
 * Creates a package name descriptor from a TOML key node.
 * @param keyNode The TOML key node.
 * @param isNameFromTable Whether the name should be extracted from the table key.
 * @returns A package name descriptor.
 */
export function createNameDescFromTomlNode(keyNode: AST.TOMLKey, isNameFromTable: boolean): PackageNameDescriptor {
  const nameNode = isNameFromTable
    ? (keyNode.parent.parent as AST.TOMLTable).key.keys[1] as AST.TOMLBare
    : keyNode.keys[0] as AST.TOMLBare;

  const nameRange = {
    start: nameNode.range[0],
    end: nameNode.range[0],
  };

  return createPackageNameDesc(nameNode.name, nameRange);
}

/**
 * Creates a package version descriptor from a TOML value node.
 * @param valueNode The TOML value node.
 * @returns A package version descriptor.
 */
export function createVersionDescFromTomlNode(
  valueNode: AST.TOMLValue
): PackageVersionDescriptor {

  const version = valueNode.value as string;

  // +1 and -1 to be inside quotes
  const versionRange = {
    start: valueNode.range[0] + 1,
    end: valueNode.range[1] - 1,
  };

  return createPackageVersionDesc(version, versionRange);
}

/**
 * Creates a package path descriptor from a TOML value node.
 * @param valueNode The TOML value node representing the path.
 * @returns A package path descriptor.
 */
export function createPathDescFromTomlNode(valueNode: any): PackagePathDescriptor {
  const path = valueNode.value as string;

  // +1 and -1 to be inside quotes
  const pathRange = {
    start: valueNode.range[0] + 1,
    end: valueNode.range[1] - 1,
  };

  return createPackagePathDescType(path, pathRange);
}

/**
 * Creates a Git descriptor from a TOML value node.
 * @param valueNode The TOML value node representing the repository URL.
 * @returns A package Git descriptor.
 */
export function createGitDescFromTomlNode(valueNode: AST.TOMLValue): PackageGitDescriptor {
  return createPackageGitDescType(valueNode.value as string);
}

/**
 * Creates a package descriptor for the project's own version from a TOML key-value node.
 * @param keyValue The TOML key-value node.
 * @returns A package descriptor for the project version.
 */
export function createProjectVersionDescFromTomlNode(keyValue: AST.TOMLKeyValue): PackageDescriptor {
  return new PackageDescriptor([
    createNameDescFromTomlNode(keyValue.key, false),
    createVersionDescFromTomlNode(keyValue.value as AST.TOMLValue),
    createProjectVersionTypeDesc()
  ]);
}