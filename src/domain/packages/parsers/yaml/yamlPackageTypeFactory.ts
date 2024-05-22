import {
  PackageDescriptorType,
  TPackageGitDescriptor,
  TPackageHostedDescriptor,
  TPackageNameDescriptor,
  TPackagePathDescriptor,
  TPackageVersionDescriptor,
  createPackageVersionDesc
} from "domain/packages";
import { Undefinable } from 'domain/utils';
import { YAMLMap } from 'yaml';
import { findPair } from 'yaml/util';

export function createNameDescFromYamlNode(keyNode: any): TPackageNameDescriptor {
  const name = keyNode.value;

  const nameRange = {
    start: keyNode.range[0],
    end: keyNode.range[0],
  };

  return {
    type: PackageDescriptorType.name,
    name,
    nameRange
  };
}

export function createVersionDescFromYamlNode(
  valueNode: any,
  isQuoteType: boolean
): TPackageVersionDescriptor {

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
  const fallbackValue = hasComment ? "#" : "";
  const version = valueNode.value || fallbackValue;

  return createPackageVersionDesc(version, versionRange, "", hasComment ? " " : "");
}

export function createPathDescFromYamlNode(
  valueNode: any,
  isQuoteType: boolean
): TPackagePathDescriptor {

  const pathRange = {
    start: valueNode.range[0],
    end: valueNode.range[1],
  };

  if (isQuoteType) {
    pathRange.start++;
    pathRange.end--;
  }

  const pathDesc: TPackagePathDescriptor = {
    type: PackageDescriptorType.path,
    path: valueNode.value,
    pathRange
  }

  return pathDesc;
}

export function createHostedDescFromYamlNode(
  valueNode: any,
  isQuoteType: boolean
): Undefinable<TPackageHostedDescriptor> {

  const map = valueNode as YAMLMap;

  // extract url from direct strings
  const isStringType = valueNode.type === "PLAIN" || isQuoteType;
  if (isStringType) {
    return {
      type: PackageDescriptorType.hosted,
      hostUrl: valueNode.value,
      hostPackageName: ""
    }
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

  return {
    type: PackageDescriptorType.hosted,
    hostPackageName,
    hostUrl
  }
}

export function createGitDescFromYamlNode(
  valueNode: any,
  isQuoteType: boolean
): Undefinable<TPackageGitDescriptor> {

  let gitUrl = "";
  let gitRef = "";
  let gitPath = "";

  // extract url from direct strings
  const isStringType = valueNode.type === "PLAIN" || isQuoteType;
  if (isStringType) {
    gitUrl = valueNode.value;
    return {
      type: PackageDescriptorType.git,
      gitUrl,
      gitRef,
      gitPath
    }
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

  return {
    type: PackageDescriptorType.git,
    gitUrl,
    gitRef,
    gitPath
  }
}