import {
  type PackageNameDescriptor,
  type PackageVersionDescriptor,
  PackageDescriptor,
  createPackageNameDesc,
  createPackageVersionDesc,
  createProjectVersionTypeDesc
} from '#domain/parsers';
import { YAMLMap } from 'yaml';

/**
 * Checks if a YAML node value is quoted.
 * @param node The YAML node.
 * @returns True if the node is quoted, otherwise false.
 */
export function isNodeQuoted(node: any) {
  return node.type === "QUOTE_SINGLE"
    || node.type === "QUOTE_DOUBLE";
}

/**
 * Creates a package name descriptor from a YAML node.
 * @param keyNode The YAML node representing the name.
 * @returns A package name descriptor.
 */
export function createNameDescFromYamlNode(keyNode: any): PackageNameDescriptor {
  const name = keyNode.value;

  const nameRange = {
    start: keyNode.range[0],
    end: keyNode.range[0],
  };

  return createPackageNameDesc(name, nameRange);
}

/**
 * Creates a package version descriptor from a YAML node.
 * @param valueNode The YAML node representing the version.
 * @param isQuoteType Whether the node value is quoted.
 * @returns A package version descriptor.
 */
export function createVersionDescFromYamlNode(
  valueNode: any,
  isQuoteType: boolean
): PackageVersionDescriptor {

  const versionRange = {
    start: valueNode.range[0],
    end: valueNode.range[1]
  };

  // +1 and -1 to be inside quotes
  if (isQuoteType) {
    versionRange.start++;
    versionRange.end--;
  }

  const hasComment = valueNode.comment !== undefined && valueNode.comment.length > 0;
  const fallbackValue = hasComment ? "*" : "";
  const hasVersion = !!valueNode.value;
  const version = hasVersion ? valueNode.value : fallbackValue;
  const append = hasVersion === false && hasComment;

  return createPackageVersionDesc(
    version,
    versionRange,
    "",
    append ? " " : ""
  );
}

/**
 * Gets a package descriptor for the project's own version from a YAML map.
 * @param map The YAML map representing the package.
 * @returns A package descriptor for the project version, or undefined.
 */
export function getPackageProjectVersionDesc(map: YAMLMap<any, any>): PackageDescriptor | undefined {
  for (const node of map.items) {
    if (node.key.value === 'version') {
      const isQuoted = isNodeQuoted(node.value);
      return new PackageDescriptor([
        createNameDescFromYamlNode(node.key),
        createVersionDescFromYamlNode(node.value, isQuoted),
        createProjectVersionTypeDesc()
      ]);
    }
  }
}