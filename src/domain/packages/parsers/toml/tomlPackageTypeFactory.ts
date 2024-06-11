import {
  PackageDescriptor,
  TPackageGitDescriptor,
  TPackageNameDescriptor,
  TPackagePathDescriptor,
  TPackageVersionDescriptor,
  createPackageGitDescType,
  createPackageNameDesc,
  createPackagePathDescType,
  createPackageVersionDesc,
  createProjectVersionTypeDesc
} from '#domain/packages';
import { AST } from 'toml-eslint-parser';
import { TOMLKeyValue, TOMLTable } from 'toml-eslint-parser/lib/ast';

export function createNameDescFromTomlNode(keyNode: AST.TOMLKey, isNameFromTable: boolean): TPackageNameDescriptor {
  const nameNode = isNameFromTable
    ? (keyNode.parent.parent as TOMLTable).key.keys[1] as AST.TOMLBare
    : keyNode.keys[0] as AST.TOMLBare;

  const nameRange = {
    start: nameNode.range[0],
    end: nameNode.range[0],
  };

  return createPackageNameDesc(nameNode.name, nameRange);
}

export function createVersionDescFromTomlNode(
  valueNode: AST.TOMLValue
): TPackageVersionDescriptor {

  const version = valueNode.value as string;

  // +1 and -1 to be inside quotes
  const versionRange = {
    start: valueNode.range[0] + 1,
    end: valueNode.range[1] - 1,
  };

  return createPackageVersionDesc(version, versionRange);
}

export function createPathDescFromTomlNode(valueNode: any): TPackagePathDescriptor {
  const path = valueNode.value as string;

  // +1 and -1 to be inside quotes
  const pathRange = {
    start: valueNode.range[0] + 1,
    end: valueNode.range[1] - 1,
  };

  return createPackagePathDescType(path, pathRange);
}

export function createGitDescFromTomlNode(valueNode: AST.TOMLValue): TPackageGitDescriptor {
  return createPackageGitDescType(valueNode.value as string);
}

export function createProjectVersionDescFromTomlNode(keyValue: TOMLKeyValue): PackageDescriptor {
  return new PackageDescriptor([
    createNameDescFromTomlNode(keyValue.key, false),
    createVersionDescFromTomlNode(keyValue.value as AST.TOMLValue),
    createProjectVersionTypeDesc()
  ]);
}