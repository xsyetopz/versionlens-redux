import {
  type PackageGitDescriptor,
  type PackageHostedDescriptor,
  type PackagePathDescriptor,
  PackageVersionDescriptor,
  createPackageGitDescType,
  createPackageHostedDescType,
  createPackagePathDescType,
  createVersionDescFromYamlNode
} from '#domain/parsers';
import type { YAMLMap } from 'yaml';
import { findPair } from 'yaml/util';

/**
 * Creates a version descriptor for a Pub dependency, handling the 'any' keyword.
 * @param valueNode The YAML node representing the version.
 * @param isQuoteType Whether the node value is quoted.
 * @returns A package version descriptor.
 */
export function createPubVersionDescFromYamlNode(
  valueNode: any,
  isQuoteType: boolean
): PackageVersionDescriptor {
  valueNode.value === 'any' && (valueNode.value = '*')
  return createVersionDescFromYamlNode(valueNode, isQuoteType)
}

/**
 * Creates a path descriptor from a YAML node.
 * @param valueNode The YAML node representing the path.
 * @param isQuoteType Whether the node value is quoted.
 * @returns A package path descriptor.
 */
export function createPathDescFromYamlNode(
  valueNode: any,
  isQuoteType: boolean
): PackagePathDescriptor {

  const pathRange = {
    start: valueNode.range[0],
    end: valueNode.range[1],
  };

  if (isQuoteType) {
    pathRange.start++;
    pathRange.end--;
  }

  return createPackagePathDescType(valueNode.value, pathRange);
}

/**
 * Creates a hosted descriptor from a YAML node.
 * @param valueNode The YAML node representing the hosted configuration.
 * @param isQuoteType Whether the node value is quoted.
 * @returns A package hosted descriptor or undefined.
 */
export function createHostedDescFromYamlNode(
  valueNode: any,
  isQuoteType: boolean
): PackageHostedDescriptor | undefined {

  const map = valueNode as YAMLMap;

  // extract url from direct strings
  const isStringType = valueNode.type === "PLAIN" || isQuoteType;
  if (isStringType) {
    return createPackageHostedDescType(valueNode.value)
  }

  // skip incomplete hosted entries
  if (map.has("url") === false) return;

  // extract the host url
  const namePair = findPair(map.items, "url");
  if (!namePair) return;

  const hostUrl = (<any>namePair.value).value;

  // extract alias package name
  let hostPackageName = "";
  if (map.has("name")) {
    const namePair = findPair(map.items, "name");
    if (!namePair) return;
    hostPackageName = (<any>namePair.value).value;
  }

  return createPackageHostedDescType(hostUrl, hostPackageName)
}

/**
 * Creates a Git descriptor from a YAML node.
 * @param valueNode The YAML node representing the Git configuration.
 * @param isQuoteType Whether the node value is quoted.
 * @returns A package Git descriptor or undefined.
 */
export function createGitDescFromYamlNode(
  valueNode: any,
  isQuoteType: boolean
): PackageGitDescriptor | undefined {

  let gitUrl = "";
  let gitRef = "";
  let gitPath = "";

  // extract url from direct strings
  const isStringType = valueNode.type === "PLAIN" || isQuoteType;
  if (isStringType) {
    gitUrl = valueNode.value;
    return createPackageGitDescType(
      gitUrl,
      gitPath,
      gitRef
    )
  }

  const map = valueNode as YAMLMap;

  // skip incomplete git entries
  if (map.has("url") === false) return;

  // extract the git url
  const namePair = findPair(map.items, "url");
  if (!namePair) return;
  gitUrl = (<any>namePair.value).value;

  // extract refs
  if (map.has("ref")) {
    const namePair = findPair(map.items, "ref");
    if (!namePair) return;
    gitRef = (<any>namePair.value).value;
  }

  // extract paths
  if (map.has("path")) {
    const namePair = findPair(map.items, "path");
    if (!namePair) return;
    gitPath = (<any>namePair.value).value;
  }

  return createPackageGitDescType(
    gitUrl,
    gitPath,
    gitRef
  );
}