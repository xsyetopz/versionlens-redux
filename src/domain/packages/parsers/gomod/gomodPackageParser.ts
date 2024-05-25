import { TPackageTextRange } from "../definitions/tPackageTextRange";
import { TPackageNameDescriptor, TPackageVersionDescriptor } from "../definitions/tPackageTypeDescriptors";
import { PackageDescriptor } from "../packageDescriptor";
import { createPackageNameDesc, createPackageVersionDesc } from "../packageDescriptorTypeUtils";

const INCOMPAT_BUILD = "+incompatible";
const PREPEND_V = "v";

export function parsePackagesGoMod(text: string): Array<PackageDescriptor> {
  const matchedDependencies: Array<PackageDescriptor> = [];
  const re = /(\S+)\s*(\sv\S+)/gd
  let match

  while ((match = re.exec(text)) !== null) {
    const packageName = match[1];
    const [packageStart] = match.indices[1];

    const version = match[2];
    const [versionStart, versionEnd] = match.indices[2];

    // create the package descriptor
    const nameDesc = createNameDesc(packageName, packageStart);
    const versionDesc = createVersionDesc(version.trim(), versionStart + 1, versionEnd);
    const packageDesc = new PackageDescriptor([nameDesc, versionDesc]);
    matchedDependencies.push(packageDesc);
  }

  return matchedDependencies;
}

export function createNameDesc(name: string, start: number): TPackageNameDescriptor {
  const nameRange: TPackageTextRange = {
    start,
    end: start
  };

  return createPackageNameDesc(name, nameRange);
}

export function createVersionDesc(version: string, start: number, end: number): TPackageVersionDescriptor {
  const versionRange = {
    start,
    end
  };

  const append = version.endsWith(INCOMPAT_BUILD) ? INCOMPAT_BUILD : "";

  return createPackageVersionDesc(
    version,
    versionRange,
    PREPEND_V,
    append
  );
}