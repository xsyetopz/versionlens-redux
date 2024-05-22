import { PackageDescriptorType } from "./definitions/ePackageDescriptorType";
import { TPackageTextRange } from "./definitions/tPackageTextRange";
import { TPackageNameDescriptor, TPackageVersionDescriptor } from "./definitions/tPackageTypeDescriptors";

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