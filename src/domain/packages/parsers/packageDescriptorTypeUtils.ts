import * as JsonC from 'jsonc-parser';
import { PackageDescriptorType } from "./definitions/ePackageDescriptorType";
import { TPackageTextRange } from "./definitions/tPackageTextRange";
import {
  TPackageNameDescriptor,
  TPackageVersionDescriptor
} from "./definitions/tPackageTypeDescriptors";
import {
  createIgnoreChangesDesc,
  createNameDescFromJsonNode,
  createParentDesc,
  createVersionDescFromJsonNode
} from "./json/jsonPackageTypeFactory";
import { PackageDescriptor } from "./packageDescriptor";

export function createPackageNameDesc(name: string, nameRange: TPackageTextRange): TPackageNameDescriptor {
  return {
    type: PackageDescriptorType.name,
    name,
    nameRange
  };
}

export function createPackageVersionDesc(
  version: string,
  versionRange: TPackageTextRange,
  versionPrepend: string = "",
  versionAppend: string = ""
): TPackageVersionDescriptor {
  return {
    type: PackageDescriptorType.version,
    version,
    versionRange,
    versionPrepend,
    versionAppend
  };
}

export function createSpecialDesc(node: JsonC.Node, path: string) {
  const nameDesc = createNameDescFromJsonNode(node);
  const versionDesc = createVersionDescFromJsonNode(node);
  const parentDesc = createParentDesc(path);
  const ignoreChangesDesc = createIgnoreChangesDesc();

  const packageDesc = new PackageDescriptor([
    nameDesc,
    versionDesc,
    parentDesc,
    ignoreChangesDesc
  ]);

  return packageDesc;
}