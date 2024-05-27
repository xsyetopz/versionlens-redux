import {
  PackageDescriptorType,
  TPackageGitDescriptor,
  TPackageIgnoreChangesDescriptor,
  TPackageNameDescriptor,
  TPackageParentDescriptor,
  TPackagePathDescriptor,
  TPackageVersionDescriptor,
  createPackageVersionDesc
} from 'domain/packages';
import * as JsonC from 'jsonc-parser';

export function createNameDescFromJsonNode(keyNode: JsonC.Node): TPackageNameDescriptor {
  const name = keyNode.value;

  const nameRange = {
    start: keyNode.offset,
    end: keyNode.offset,
  };

  return {
    type: PackageDescriptorType.name,
    name,
    nameRange
  };
}

export function createVersionDescFromJsonNode(valueNode: any): TPackageVersionDescriptor {
  // +1 and -1 to be inside quotes
  const versionRange = {
    start: valueNode.offset + 1,
    end: valueNode.offset + valueNode.length - 1,
  };

  // Handle packageManager field (packageManager@version)
  let { value: version } = valueNode;
  if (/^[\w]+@[\w-.]+$/.test(version)) {
    const versionSplitIndex = version.indexOf("@");
    if (versionSplitIndex !== -1) {
      version = version.substring(versionSplitIndex + 1);
      versionRange.start += versionSplitIndex + 1;
    }
  }

  return createPackageVersionDesc(version, versionRange);
}

export function createPathDescFromJsonNode(
  valueNode: any
): TPackagePathDescriptor {

  // +1 and -1 to be inside quotes
  const pathRange = {
    start: valueNode.offset + 1,
    end: valueNode.offset + valueNode.length - 1,
  };

  return {
    type: PackageDescriptorType.path,
    path: valueNode.value,
    pathRange: pathRange
  }
}

export function createRepoDescFromJsonNode(
  valueNode: any
): TPackageGitDescriptor {

  return {
    type: PackageDescriptorType.git,
    gitUrl: valueNode.value,
    gitPath: "",
    gitRef: ""
  }
}

export function createParentDesc(path: string): TPackageParentDescriptor {
  return {
    type: PackageDescriptorType.parent,
    path
  }
}

export function createIgnoreChangesDesc(): TPackageIgnoreChangesDescriptor {
  return { type: PackageDescriptorType.ignoreChanges }
}