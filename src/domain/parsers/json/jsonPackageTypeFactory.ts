import {
  PackageDescriptor,
  TPackageGitDescriptor,
  TPackageNameDescriptor,
  TPackagePathDescriptor,
  TPackageVersionDescriptor,
  createPackageGitDescType,
  createPackageNameDesc,
  createPackageParentDescType,
  createPackagePathDescType,
  createPackageVersionDesc,
  createProjectVersionTypeDesc
} from '#domain/parsers';
import * as JsonC from 'jsonc-parser';

export function createNameDescFromJsonNode(keyNode: JsonC.Node): TPackageNameDescriptor {
  const name = keyNode.value;

  const nameRange = {
    start: keyNode.offset,
    end: keyNode.offset,
  };

  return createPackageNameDesc(name, nameRange);
}

export function createVersionDescFromJsonNode(valueNode: JsonC.Node): TPackageVersionDescriptor {
  // +1 and -1 to be inside quotes
  const versionRange = {
    start: valueNode.offset + 1,
    end: valueNode.offset + valueNode.length - 1,
  };

  const { value: version } = valueNode;

  return createPackageVersionDesc(version, versionRange);
}

export function createPathDescFromJsonNode(valueNode: JsonC.Node): TPackagePathDescriptor {
  // +1 and -1 to be inside quotes
  const pathRange = {
    start: valueNode.offset + 1,
    end: valueNode.offset + valueNode.length - 1,
  };

  return createPackagePathDescType(valueNode.value, pathRange);
}

export function createRepoDescFromJsonNode(valueNode: JsonC.Node): TPackageGitDescriptor {
  return createPackageGitDescType(valueNode.value);
}

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